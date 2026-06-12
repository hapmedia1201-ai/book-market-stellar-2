# Textbook Rental Hub

## Project Title
Textbook Rental Hub

## Project Description
Textbook Rental Hub is a decentralized smart contract platform built on Soroban (Stellar blockchain) that lets students rent individual chapters of textbooks instead of buying entire books. Book owners list their textbooks chapter by chapter with a daily token price. When a renter pays, tokens transfer directly and instantly to the book owner — no escrow, no platform cut, no middleman.

## Project Vision
The vision of Textbook Rental Hub is to make educational materials more accessible and affordable by letting students pay only for the chapters they actually need. By settling payments peer-to-peer on Stellar, it removes the need for any trusted third party to hold funds, giving both owners and renters a transparent and trustless experience.

## Key Features
- **Chapter-Level Rentals:** Each chapter of a book is listed and rented independently with its own daily price.
- **Direct Token Transfer:** Rental payment goes straight from renter to owner at the moment of rental — no escrow, no delay.
- **Flexible Duration:** Renters choose how many days they need each chapter.
- **Availability Tracking:** A chapter is marked unavailable while rented and reopens automatically when returned.
- **Price Management:** Owners can update the price of any chapter that is not currently rented.
- **Early Return:** Renters or owners can end a rental early, immediately freeing the chapter for the next renter.
- **On-Chain Records:** Every rental — book ID, chapter, renter, owner, duration, amount paid — is stored on-chain for full auditability.
- **User Stats:** Each address tracks total tokens earned, total tokens spent, rentals given, and rentals taken.

## Usage Instructions
1. **Deploy & Initialize:** Deploy the contract, then call `initialize` with an admin address and a SEP-41 token contract address.
2. **List a Book:** Owner calls `list_book` with book details and arrays of chapter numbers, chapter titles, and prices per day.
3. **Rent a Chapter:** Renter approves the token amount, then calls `rent_chapter` with the book ID, chapter number, and number of days. Tokens transfer instantly to the owner.
4. **End a Rental:** Renter or owner calls `end_rental` when done. The chapter becomes available again for the next renter.
5. **Update Price:** Owner calls `update_chapter_price` on any chapter that is not currently rented.
6. **Toggle Book:** Owner calls `toggle_book` to activate or deactivate the entire listing.
7. **Query:** Anyone can call `get_available_chapters`, `search_by_grade`, `search_by_subject`, `get_rental`, or `get_user` to browse and verify data on-chain.

## Future Scope
- **Late Return Penalty:** Automatically charge extra tokens if a renter holds a chapter past the agreed end ledger.
- **Partial Refund on Early Return:** Return unused days' worth of tokens if a renter returns early.
- **Chapter Bundles:** Let owners offer discounted pricing for renting multiple chapters at once.
- **Rating System:** On-chain ratings for both owners and renters after each completed rental.
- **Frontend Dashboard:** Web interface with Freighter wallet integration for browsing, listing, and managing rentals.
- **Indexer & Notifications:** Off-chain event indexer to alert owners when a chapter is returned and ready to re-rent.
- **Multi-token Support:** Accept multiple SEP-41 tokens as payment currency.

## Technology Stack
- Rust and Soroban SDK v22 for smart contract development.
- Stellar blockchain for decentralized, immutable state and peer-to-peer token settlement.
- SEP-41 token standard for on-chain payments.
- @stellar/stellar-sdk and Freighter wallet for frontend integration.

## Contract Functions

| Function | Description |
|---|---|
| `initialize` | Set up contract with admin and token address |
| `list_book` | List a book with chapters, titles, and prices per day |
| `rent_chapter` | Rent a chapter for N days; tokens transfer instantly to owner |
| `end_rental` | End an active rental; chapter becomes available again |
| `update_chapter_price` | Update the daily price of an available chapter |
| `toggle_book` | Activate or deactivate an entire book listing |
| `get_book` | Get full details of a book including all chapters |
| `get_rental` | Get details of a specific rental record |
| `get_available_chapters` | List chapters currently available to rent in a book |
| `search_by_grade` | Filter active books by school grade (1–12) |
| `search_by_subject` | Filter active books by subject |
| `get_user` | Get rental stats and token totals for a user |
| `total_books` | Get total number of books listed |

## Token Flow

```
Renter approves token → rent_chapter()
    └── 100% → Owner (instant, direct transfer)

No platform fee. No escrow. No intermediary.
```

## How to Run

1. Clone:
   ```bash
   git clone https://github.com/yourname/textbook-rental.git
   cd textbook-rental
   ```

2. Build:
   ```bash
   cd contracts/textbook-rental
   stellar contract build
   ```

3. Test:
   ```bash
   cargo test
   ```

4. Deploy to Testnet:
   ```bash
   stellar contract deploy \
     --wasm target/wasm32-unknown-unknown/release/textbook_rental.wasm \
     --source-account student \
     --network testnet
   ```

5. Initialize:
   ```bash
   stellar contract invoke \
     --id <CONTRACT_ID> \
     -- initialize \
     --admin <ADMIN_ADDRESS> \
     --token_contract <TOKEN_ADDRESS>
   ```

6. Frontend:
   ```bash
   cd frontend && npx serve .
   ```

## Contract Detail
- Network: Stellar Testnet
- **Contract ID**: `CASPF5WQYBTSFHV56JDFRII4L4HVHSWYMYQ677MQISDFJ7PRAVZILMW7`
- **Transaction**: https://stellar.expert/explorer/testnet/tx/886d9e29a8219561a8774bf36b3aa8e877688dbd503891eeddde1d5c12b61bc3

## Contribution
Contributions are welcome from blockchain developers and educators. Fork the repository and submit a pull request to help improve the platform.

## License
This project is licensed under the MIT License.

## Team
- Nguyen Minh Tri | ntri4747@gmail.com | Saigontech 2026
