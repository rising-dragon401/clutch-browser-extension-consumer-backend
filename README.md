# clutch-browser-extension-backend
Backend store and APIs

[![BackendTests](https://github.com/clutch-wallet/clutch-browser-extension-backend/actions/workflows/linux-ci.yml/badge.svg)](https://github.com/clutch-wallet/clutch-browser-extension-backend/actions/workflows/linux-ci.yml)

### Tech Stack

* [Axum web framework](https://github.com/tokio-rs/axum)
* [SeaOrm database ORM](https://www.sea-ql.org/SeaORM/docs/index/)
* [Refinery migrations](https://github.com/rust-db/refinery)
* [Config](https://github.com/mehcode/config-rs)
* [Insta testing framework](https://insta.rs/)

### Setup

```
cargo install cargo-insta
cargo install sea-orm-cli
```

### Config

Configuration is done using the Rust Config package and there are 3 config files in the config folder in the root of the project:

* dev.toml
* prod.toml
* test.toml

The config is specified via an environment variable `RUN_MODE` set to either dev,test or prod. If no variable is set it falls back to the dev config. You can supply a local.toml file that you don't check into source control to have a local config. You can override any config with an environment variable that is prefixed with `APP_` aso.

### Run the app

* will auto create the database if doesn't exist and run the migrations

```
cargo run 
```

#### Testing

Integration tests use insta - but all tests including insta ones are run when running

```
cargo test
```

* [Insta Quickstart](https://insta.rs/docs/quickstart/) 

```
cargo insta test          // to run insta tests
cargo insta review        // to review
cargo insta test --review // all in one
```

#### Local Checks

* before pushing run `./check.sh` which will run the format & lint checks and then the tests

### Infrasructure ToDo

- [ ] implement utoipa swagger doc (https://github.com/juhaku/utoipa/blob/master/examples/todo-axum/src/main.rs)
- [ ] Continuous deployment - build Docker image and push to Dockerhub and auto deploy to the cloud (Hetzer?)

### Implementation Notes

models:

Guardian
- id
- email (of guardian)
- account_id (optional - if clutch account)
- wallet_address (optional - if no clutch account)

Guardian Nomination
- id 
- email (of potential guardian)
- guardian_id (source depends on case e.g 1. guardian exists, create from clutch user, create form external)
- account_id (of user who nominated)
- status (pending, accepted, rejected)

AccountGuardians
- guardian_id
- account_id
- status (Active/Available)

(1, acc1, ACTIVE)
(1, acc2, AVAILABLE)

AccountGuardianSettings
- account_id
- signing_rule (ONE_OF_ONE, etc)
- timing ??

Create nomination:
1. nominate with email 
2. if a guardian already exists with the email - add guardian_id to nomination

### Guardian System Overview

On new wallet creation we automatically assign Clutch as the guardian in a 1/1 signer configuration. 

The user can then change the Guardian settings and add/remove guardians later.

The user can nominate guardians to add to their list of available guardians and then they can change their active configuration from 1/1 to their desired amount as long as they have enough guardians in their available list. 

A user an also delete a guardian from their available list. If they want to remove an active guardian they have to first remove the guardian from their active configuration and then delete the guardian from their availble list.

If you are a guardian you can see a list of all the accounts you are a guardian for and you can request to no longer be a guardian for any of them. This will email the account holder telling them you are withdrawing from being their guardian and will remove them from the users available list - if they are active in a users list - it will also remove them from the active list and attempt to set a new configuration for the user - if can't then it will fall back to having 1/1 with Clutch as the guardian. 

#### Use Cases

**Single Guardian**
The user wants to keep a single Guardian configuration but change the Guardian from Clutch to someone else:

1. User will nominate a guardian by supplying an email address for the guardian. The guardian will get an email asking them to create a Clutch wallet (using the same email they were nominated on) and then in their walet they will see a guardian request they can accept or refuse. If the guardian already has an account they will get an email asking them to log into their account and decide if they want to accept the guardian role.

2. If the guardian accepts then the user can remove the current active guardian (Clutch) and add new guardian

3. Clutch will remain in the available list permanently and can't be deleted

**Multiple Guardians**
The user wants to add more than one guardian so they must nominate more guardians as per the process described in single guardian. 

1. User nominates as many guardians as they like up to a maximum of 3
1. Depending on how many guardians the user has they can change the guardian configuration from 1/1 to 2/3 - should reject if they don't have enough guardians to support the configuration. 

### Smart Contract Integration

* Figure out how smart contracts work with the code 
* Move the blockchain integration to the backend so the front end just receives things to sign
* For recovery - figure out how the guardian signatures work (a multisig?) 
* When clutch is the guardian should we automate recovery or have someone manually accept/sign?

### Emails ToDo
- [ ] Guardian nominated email
- [ ] Guardian accept / reject email

### Routes ToDo

- [x] [Email verification](https://github.com/clutch-wallet/clutch-browser-extension-backend/issues/1)
  - [x] POST /email/verify
 - [x] User Wallet Management
  - [x] create - POST /createwallet
  - [x] send transaction - POST /send
  - [x] receive transaction - POST /receive
  - [x] swap transdaction - POST/swap
- [x] Account Management
  - [x] create - POST /accounts
  - [x] retrieve all - GET /accounts
  - [x] retrieve by address - GET /accounts?(wallet_address|eoa_address|email)=
  - [x] update - PUT /accounts
- [x] Account Guardian Nomination (for authenticated user account)
  - [x] create - POST /accounts/nominations
  - [x] retrieve all - GET /accounts/nominations
  - [x] retrieve all by filter - GET /accounts/nominations?(status|nomination_id|email)=
  - [x] delete - DELETE /accounts/nomimations/:nomination_id (unless status accepted/rejected)
- [x] Account Guardian Nomination (for guardians with clutch account)
  - [x] retrieve all - GET /guardian/accounts
  - [x] retrieve by filter - GET /guardian/accounts?account_id=
  - [x] retrieve nominations - GET /guardian/nominations
  - [x] retrieve nominations by filter - GET /guardian/nominations?(status|nomination_id)=
  - [x] update - PUT /guardian/nominations/:nomination_id/accept or reject
- [x] Account Guardian Management (for authenticated user account)
  - [x] retrieve all - GET /accounts/guardians
  - [x] retrieve by filter - GET /accounts/guardians?(status|guardian_id)=
  - [x] delete - DELETE /accounts/guardians (remove guardian from account)  
- [x] Account Guardian Settings (for authenticated user account)
  - [x] retrieve - GET /accounts/guardian_settings
  - [x] update - PUT /accounts/guardian_settings
- [ ] Guardian Management (for external guardians)
   // endpoint to add a wallet address 
  - [ ] retrieve all - GET /guardian/accounts
  - [ ] retrieve by id - GET /guardian/accounts/:account_id
  - [ ] retrieve nominations - GET /guardian/nominations
  - [ ] retrieve nominations by filter - GET /guardian/nominations?(status)=
  - [ ] retrieve by nomination id - GET /guardian/nomination/:nomination_id
  - [ ] update - PUT /guardian/nominations/:nomination_id/accept or reject 
  - [ ] delete - DELETE /guardian/accounts
- [ ] Recovery
  - [ ] create - POST /recovery-record
  - [ ] retrieve by filter - GET /recovery-records?state=(id|pending|signed)
  - [ ] update - PUT /recovery-record (state changes also e.g. Finish)
  - [ ] delete - DELETE /recovery-record/id
