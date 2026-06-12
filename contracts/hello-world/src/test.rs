#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype,
    token, Address, Env, String, Vec,
    symbol_short,
};

// ─────────────────────────────────────────────
// DATA TYPES
// ─────────────────────────────────────────────

/// Một chương sách có thể cho thuê độc lập
#[contracttype]
#[derive(Clone)]
pub struct Chapter {
    pub chapter_num: u32,       // số chương: 1, 2, 3...
    pub title: String,          // tên chương
    pub price_per_day: i128,    // giá token thuê / ngày
    pub is_available: bool,     // còn cho thuê không
}

/// Thông tin quyển sách được đăng cho thuê
#[contracttype]
#[derive(Clone)]
pub struct Book {
    pub id: u64,
    pub title: String,
    pub subject: String,        // môn học
    pub grade: u32,             // lớp 1-12
    pub owner: Address,
    pub condition: u32,         // 1-5
    pub chapters: Vec<Chapter>, // danh sách chương
    pub is_active: bool,        // chủ sách có đang hoạt động không
}

/// Một hợp đồng thuê chương sách
#[contracttype]
#[derive(Clone)]
pub struct Rental {
    pub id: u64,
    pub book_id: u64,
    pub chapter_num: u32,
    pub renter: Address,
    pub owner: Address,
    pub price_per_day: i128,
    pub duration_days: u32,     // số ngày thuê
    pub total_paid: i128,       // tổng token đã thanh toán
    pub start_ledger: u32,      // ledger bắt đầu
    pub end_ledger: u32,        // ledger kết thúc (ước tính)
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct User {
    pub address: Address,
    pub books_listed: u64,
    pub total_earned: i128,     // tổng token nhận được từ cho thuê
    pub total_spent: i128,      // tổng token đã trả để thuê
    pub rentals_given: u64,     // số lần cho thuê
    pub rentals_taken: u64,     // số lần đi thuê
}

#[contracttype]
pub enum DataKey {
    BookCount,
    Book(u64),
    RentalCount,
    Rental(u64),
    User(Address),
    TokenContract,
    Admin,
}

// Stellar Testnet: ~1 ledger / 5 giây → 1 ngày ≈ 17280 ledgers
const LEDGERS_PER_DAY: u32 = 17_280;

// ─────────────────────────────────────────────
// CONTRACT
// ─────────────────────────────────────────────

#[contract]
pub struct TextbookRental;

#[contractimpl]
impl TextbookRental {

    // ── KHỞI TẠO ───────────────────────────────

    pub fn initialize(env: Env, admin: Address, token_contract: Address) {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract da duoc khoi tao");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TokenContract, &token_contract);
        env.storage().instance().set(&DataKey::BookCount, &0u64);
        env.storage().instance().set(&DataKey::RentalCount, &0u64);
    }

    // ── ĐĂNG SÁCH CHO THUÊ ─────────────────────
    // chapters: Vec<(chapter_num, chapter_title, price_per_day)>

    pub fn list_book(
        env: Env,
        owner: Address,
        title: String,
        subject: String,
        grade: u32,
        condition: u32,
        chapter_nums: Vec<u32>,
        chapter_titles: Vec<String>,
        chapter_prices: Vec<i128>,
    ) -> u64 {
        owner.require_auth();

        if grade < 1 || grade > 12 {
            panic!("grade phai tu 1 den 12");
        }
        if condition < 1 || condition > 5 {
            panic!("condition phai tu 1 den 5");
        }
        if chapter_nums.len() == 0 {
            panic!("Phai co it nhat 1 chuong");
        }
        if chapter_nums.len() != chapter_titles.len()
            || chapter_nums.len() != chapter_prices.len()
        {
            panic!("So luong chapter_nums, titles, prices phai bang nhau");
        }

        // Build danh sách chương
        let mut chapters: Vec<Chapter> = Vec::new(&env);
        for i in 0..chapter_nums.len() {
            let price = chapter_prices.get(i).unwrap();
            if price <= 0 {
                panic!("Gia thue phai lon hon 0");
            }
            chapters.push_back(Chapter {
                chapter_num: chapter_nums.get(i).unwrap(),
                title: chapter_titles.get(i).unwrap(),
                price_per_day: price,
                is_available: true,
            });
        }

        let book_id: u64 = env
            .storage().instance()
            .get(&DataKey::BookCount)
            .unwrap_or(0u64) + 1;

        let book = Book {
            id: book_id,
            title,
            subject,
            grade,
            owner: owner.clone(),
            condition,
            chapters,
            is_active: true,
        };

        env.storage().instance().set(&DataKey::Book(book_id), &book);
        env.storage().instance().set(&DataKey::BookCount, &book_id);
        Self::update_user_stats(&env, &owner, 1, 0, 0, 0, 0);

        env.events().publish(
            (symbol_short!("listed"), owner),
            book_id,
        );

        book_id
    }

    // ── THUÊ MỘT CHƯƠNG ────────────────────────
    // Token chuyển thẳng đến owner ngay khi gọi hàm
    // Renter phải approve token trước

    pub fn rent_chapter(
        env: Env,
        renter: Address,
        book_id: u64,
        chapter_num: u32,
        duration_days: u32,
    ) -> u64 {
        renter.require_auth();

        if duration_days == 0 {
            panic!("Thoi gian thue phai it nhat 1 ngay");
        }

        let mut book: Book = env
            .storage().instance()
            .get(&DataKey::Book(book_id))
            .expect("Khong tim thay sach");

        if !book.is_active {
            panic!("Sach nay khong con cho thue");
        }
        if book.owner == renter {
            panic!("Ban khong the thue sach cua chinh minh");
        }

        // Tìm chương
        let chapter_idx = Self::find_chapter_idx(&book.chapters, chapter_num);
        let mut chapter = book.chapters.get(chapter_idx).unwrap();

        if !chapter.is_available {
            panic!("Chuong nay dang duoc thue boi nguoi khac");
        }

        let total_payment = chapter.price_per_day * duration_days as i128;

        // Lấy token contract
        let token_contract: Address = env
            .storage().instance()
            .get(&DataKey::TokenContract)
            .expect("Chua khoi tao token contract");

        let token = token::Client::new(&env, &token_contract);

        // Chuyển token thẳng từ renter → owner
        token.transfer(&renter, &book.owner, &total_payment);

        // Đánh dấu chương đang được thuê
        chapter.is_available = false;
        book.chapters.set(chapter_idx, chapter.clone());
        env.storage().instance().set(&DataKey::Book(book_id), &book);

        // Tạo rental record
        let rental_id: u64 = env
            .storage().instance()
            .get(&DataKey::RentalCount)
            .unwrap_or(0u64) + 1;

        let current_ledger = env.ledger().sequence();
        let rental = Rental {
            id: rental_id,
            book_id,
            chapter_num,
            renter: renter.clone(),
            owner: book.owner.clone(),
            price_per_day: chapter.price_per_day,
            duration_days,
            total_paid: total_payment,
            start_ledger: current_ledger,
            end_ledger: current_ledger + (duration_days * LEDGERS_PER_DAY),
            is_active: true,
        };

        env.storage().instance().set(&DataKey::Rental(rental_id), &rental);
        env.storage().instance().set(&DataKey::RentalCount, &rental_id);

        // Cập nhật stats
        Self::update_user_stats(&env, &book.owner, 0, 1, 0, total_payment, 0);
        Self::update_user_stats(&env, &renter, 0, 0, 1, 0, total_payment);

        env.events().publish(
            (symbol_short!("rented"), renter),
            (rental_id, book_id, chapter_num, total_payment),
        );

        rental_id
    }

    // ── TRẢ SÁCH / KẾT THÚC THUÊ ───────────────
    // Ai cũng có thể gọi sau khi hết hạn
    // Chỉ renter hoặc owner mới có thể kết thúc sớm

    pub fn end_rental(env: Env, caller: Address, rental_id: u64) {
        caller.require_auth();

        let mut rental: Rental = env
            .storage().instance()
            .get(&DataKey::Rental(rental_id))
            .expect("Khong tim thay rental");

        if !rental.is_active {
            panic!("Rental nay da ket thuc");
        }

        // Chỉ renter hoặc owner mới được kết thúc sớm
        let current_ledger = env.ledger().sequence();
        let is_expired = current_ledger >= rental.end_ledger;

        if !is_expired && caller != rental.renter && caller != rental.owner {
            panic!("Chi renter hoac owner moi co the ket thuc som");
        }

        rental.is_active = false;
        env.storage().instance().set(&DataKey::Rental(rental_id), &rental);

        // Mở lại chương cho thuê tiếp
        let mut book: Book = env
            .storage().instance()
            .get(&DataKey::Book(rental.book_id))
            .expect("Khong tim thay sach");

        let chapter_idx = Self::find_chapter_idx(&book.chapters, rental.chapter_num);
        let mut chapter = book.chapters.get(chapter_idx).unwrap();
        chapter.is_available = true;
        book.chapters.set(chapter_idx, chapter);
        env.storage().instance().set(&DataKey::Book(rental.book_id), &book);

        env.events().publish(
            (symbol_short!("returned"), caller),
            rental_id,
        );
    }

    // ── CẬP NHẬT GIÁ CHƯƠNG ────────────────────

    pub fn update_chapter_price(
        env: Env,
        owner: Address,
        book_id: u64,
        chapter_num: u32,
        new_price: i128,
    ) {
        owner.require_auth();

        if new_price <= 0 {
            panic!("Gia phai lon hon 0");
        }

        let mut book: Book = env
            .storage().instance()
            .get(&DataKey::Book(book_id))
            .expect("Khong tim thay sach");

        if book.owner != owner {
            panic!("Ban khong co quyen sua gia sach nay");
        }

        let idx = Self::find_chapter_idx(&book.chapters, chapter_num);
        let mut chapter = book.chapters.get(idx).unwrap();

        if !chapter.is_available {
            panic!("Chuong dang duoc thue, khong the doi gia");
        }

        chapter.price_per_day = new_price;
        book.chapters.set(idx, chapter);
        env.storage().instance().set(&DataKey::Book(book_id), &book);
    }

    // ── TẮT / BẬT SÁCH ─────────────────────────

    pub fn toggle_book(env: Env, owner: Address, book_id: u64) {
        owner.require_auth();

        let mut book: Book = env
            .storage().instance()
            .get(&DataKey::Book(book_id))
            .expect("Khong tim thay sach");

        if book.owner != owner {
            panic!("Ban khong co quyen chinh sua sach nay");
        }

        book.is_active = !book.is_active;
        env.storage().instance().set(&DataKey::Book(book_id), &book);
    }

    // ── QUERY ───────────────────────────────────

    pub fn get_book(env: Env, book_id: u64) -> Book {
        env.storage().instance()
            .get(&DataKey::Book(book_id))
            .expect("Khong tim thay sach")
    }

    pub fn get_rental(env: Env, rental_id: u64) -> Rental {
        env.storage().instance()
            .get(&DataKey::Rental(rental_id))
            .expect("Khong tim thay rental")
    }

    pub fn search_by_grade(env: Env, grade: u32) -> Vec<Book> {
        Self::filter_books(&env, |b| b.grade == grade && b.is_active)
    }

    pub fn search_by_subject(env: Env, subject: String) -> Vec<Book> {
        Self::filter_books(&env, |b| b.subject == subject && b.is_active)
    }

    pub fn get_available_chapters(env: Env, book_id: u64) -> Vec<Chapter> {
        let book: Book = env
            .storage().instance()
            .get(&DataKey::Book(book_id))
            .expect("Khong tim thay sach");

        let mut available: Vec<Chapter> = Vec::new(&env);
        for i in 0..book.chapters.len() {
            let ch = book.chapters.get(i).unwrap();
            if ch.is_available {
                available.push_back(ch);
            }
        }
        available
    }

    pub fn get_user(env: Env, address: Address) -> User {
        env.storage().instance()
            .get(&DataKey::User(address.clone()))
            .unwrap_or(User {
                address,
                books_listed: 0,
                total_earned: 0,
                total_spent: 0,
                rentals_given: 0,
                rentals_taken: 0,
            })
    }

    pub fn total_books(env: Env) -> u64 {
        env.storage().instance()
            .get(&DataKey::BookCount)
            .unwrap_or(0u64)
    }

    // ─────────────────────────────────────────
    // INTERNAL HELPERS
    // ─────────────────────────────────────────

    fn find_chapter_idx(chapters: &Vec<Chapter>, chapter_num: u32) -> u32 {
        for i in 0..chapters.len() {
            if chapters.get(i).unwrap().chapter_num == chapter_num {
                return i;
            }
        }
        panic!("Khong tim thay chuong nay trong sach");
    }

    fn filter_books<F>(env: &Env, predicate: F) -> Vec<Book>
    where
        F: Fn(&Book) -> bool,
    {
        let total: u64 = env
            .storage().instance()
            .get(&DataKey::BookCount)
            .unwrap_or(0u64);

        let mut results: Vec<Book> = Vec::new(env);
        for id in 1..=total {
            if let Some(book) = env
                .storage().instance()
                .get::<DataKey, Book>(&DataKey::Book(id))
            {
                if predicate(&book) {
                    results.push_back(book);
                }
            }
        }
        results
    }

    fn update_user_stats(
        env: &Env,
        address: &Address,
        listed_delta: u64,
        given_delta: u64,
        taken_delta: u64,
        earned_delta: i128,
        spent_delta: i128,
    ) {
        let mut user: User = env
            .storage().instance()
            .get(&DataKey::User(address.clone()))
            .unwrap_or(User {
                address: address.clone(),
                books_listed: 0,
                total_earned: 0,
                total_spent: 0,
                rentals_given: 0,
                rentals_taken: 0,
            });

        user.books_listed   += listed_delta;
        user.rentals_given  += given_delta;
        user.rentals_taken  += taken_delta;
        user.total_earned   += earned_delta;
        user.total_spent    += spent_delta;

        env.storage().instance()
            .set(&DataKey::User(address.clone()), &user);
    }
}

// ─────────────────────────────────────────────
// TESTS
// ─────────────────────────────────────────────

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{token, Env};
    use soroban_sdk::token::StellarAssetClient;

    fn setup(env: &Env) -> (TextbookRentalClient, Address, Address) {
        let admin = Address::generate(env);
        let token_id = env.register_stellar_asset_contract_v2(admin.clone());
        let stellar_asset = StellarAssetClient::new(env, &token_id.address());
        stellar_asset.mint(&admin, &1_000_000_000);

        let contract_id = env.register_contract(None, TextbookRental);
        let client = TextbookRentalClient::new(env, &contract_id);
        client.initialize(&admin, &token_id.address());

        (client, admin, token_id.address())
    }

    #[test]
    fn test_list_and_rent_chapter() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, token_contract) = setup(&env);

        let owner  = Address::generate(&env);
        let renter = Address::generate(&env);

        // Mint token cho renter
        let stellar_asset = StellarAssetClient::new(&env, &token_contract);
        stellar_asset.mint(&renter, &10_000);

        // Đăng sách với 3 chương
        let book_id = client.list_book(
            &owner,
            &String::from_str(&env, "Toan 10 - Canh Dieu"),
            &String::from_str(&env, "Toan"),
            &10u32,
            &4u32,
            &Vec::from_array(&env, [1u32, 2u32, 3u32]),
            &Vec::from_array(&env, [
                String::from_str(&env, "Menh de va phep suy luan"),
                String::from_str(&env, "Tap hop"),
                String::from_str(&env, "Logic toan hoc"),
            ]),
            &Vec::from_array(&env, [100i128, 120i128, 80i128]),
        );

        assert_eq!(book_id, 1);

        // Thuê chương 2 trong 3 ngày → 120 * 3 = 360 token
        let rental_id = client.rent_chapter(&renter, &book_id, &2u32, &3u32);

        // Kiểm tra token đã chuyển thẳng cho owner
        let token_client = token::Client::new(&env, &token_contract);
        assert_eq!(token_client.balance(&owner), 360);
        assert_eq!(token_client.balance(&renter), 10_000 - 360);

        // Kiểm tra rental record
        let rental = client.get_rental(&rental_id);
        assert_eq!(rental.total_paid, 360);
        assert_eq!(rental.duration_days, 3);
        assert_eq!(rental.is_active, true);

        // Chương 2 không còn available
        let available = client.get_available_chapters(&book_id);
        assert_eq!(available.len(), 2); // còn chương 1 và 3

        // Stats
        let owner_stats = client.get_user(&owner);
        assert_eq!(owner_stats.total_earned, 360);
        assert_eq!(owner_stats.rentals_given, 1);

        let renter_stats = client.get_user(&renter);
        assert_eq!(renter_stats.total_spent, 360);
        assert_eq!(renter_stats.rentals_taken, 1);
    }

    #[test]
    fn test_end_rental_reopens_chapter() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, token_contract) = setup(&env);

        let owner  = Address::generate(&env);
        let renter = Address::generate(&env);

        let stellar_asset = StellarAssetClient::new(&env, &token_contract);
        stellar_asset.mint(&renter, &10_000);

        let book_id = client.list_book(
            &owner,
            &String::from_str(&env, "Ly 11"),
            &String::from_str(&env, "Ly"),
            &11u32,
            &3u32,
            &Vec::from_array(&env, [1u32]),
            &Vec::from_array(&env, [String::from_str(&env, "Dong dien")]),
            &Vec::from_array(&env, [200i128]),
        );

        let rental_id = client.rent_chapter(&renter, &book_id, &1u32, &2u32);

        // Renter trả sách sớm
        client.end_rental(&renter, &rental_id);

        let rental = client.get_rental(&rental_id);
        assert_eq!(rental.is_active, false);

        // Chương 1 mở lại
        let available = client.get_available_chapters(&book_id);
        assert_eq!(available.len(), 1);
    }

    #[test]
    fn test_update_chapter_price() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = setup(&env);

        let owner = Address::generate(&env);

        let book_id = client.list_book(
            &owner,
            &String::from_str(&env, "Hoa 12"),
            &String::from_str(&env, "Hoa"),
            &12u32,
            &5u32,
            &Vec::from_array(&env, [1u32]),
            &Vec::from_array(&env, [String::from_str(&env, "Hidrocacbon")]),
            &Vec::from_array(&env, [150i128]),
        );

        client.update_chapter_price(&owner, &book_id, &1u32, &250i128);

        let book = client.get_book(&book_id);
        assert_eq!(book.chapters.get(0).unwrap().price_per_day, 250);
    }
}
