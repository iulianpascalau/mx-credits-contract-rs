use multiversx_sc_scenario::imports::*;

mod requests_stub {
    use multiversx_sc::imports::*;

    #[multiversx_sc::contract]
    pub trait RequestsStub {
        #[init]
        fn init(&self, rate: BigUint) {
            self.rate().set(rate);
        }

        #[payable("EGLD")]
        #[endpoint(addRequests)]
        fn add_requests(&self, id: u64) {
            let payment = self.call_value().egld();
            let rate = self.rate().get();
            let one_egld = BigUint::from(1_000_000_000_000_000_000u64);
            let val = (payment.clone_value() * rate) / one_egld;
            self.requests(&id).update(|v| *v += val);
        }

        #[view(getRequests)]
        fn get_requests(&self, id: u64) -> BigUint {
            self.requests(&id).get()
        }

        #[storage_mapper("requests")]
        fn requests(&self, id: &u64) -> SingleValueMapper<BigUint>;

        #[storage_mapper("rate")]
        fn rate(&self) -> SingleValueMapper<BigUint>;
    }
}

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.set_current_dir_from_workspace("");
    blockchain.register_contract("file:scenarios/requests_contract/requests/requests.wasm", requests_stub::ContractBuilder);
    blockchain.register_contract("file:output/credits.wasm", credits::ContractBuilder);

    blockchain
}

#[test]
fn add_credits_accumulation_rs() {
    world().run("scenarios/add_credits_accumulation.scen.json");
}

#[test]
fn add_credits_multiple_rs() {
    world().run("scenarios/add_credits_multiple.scen.json");
}

#[test]
fn add_credits_single_rs() {
    world().run("scenarios/add_credits_single.scen.json");
}

#[test]
fn add_credits_when_paused_rs() {
    world().run("scenarios/add_credits_when_paused.scen.json");
}

#[test]
fn change_rate_non_owner_rs() {
    world().run("scenarios/change_rate_non_owner.scen.json");
}

#[test]
fn change_rate_valid_rs() {
    world().run("scenarios/change_rate_valid.scen.json");
}

#[test]
fn change_rate_zero_rs() {
    world().run("scenarios/change_rate_zero.scen.json");
}

#[test]
fn full_workflow_rs() {
    world().run("scenarios/full_workflow.scen.json");
}

#[test]
fn get_credits_existing_rs() {
    world().run("scenarios/get_credits_existing.scen.json");
}

#[test]
fn get_credits_nonexistent_rs() {
    world().run("scenarios/get_credits_nonexistent.scen.json");
}

#[test]
fn init_valid_rs() {
    world().run("scenarios/init_valid.scen.json");
}

#[test]
fn init_zero_rs() {
    world().run("scenarios/init_zero.scen.json");
}

#[test]
fn pause_already_paused_rs() {
    world().run("scenarios/pause_already_paused.scen.json");
}

#[test]
fn pause_non_owner_rs() {
    world().run("scenarios/pause_non_owner.scen.json");
}

#[test]
fn pause_success_rs() {
    world().run("scenarios/pause_success.scen.json");
}

#[test]
fn pause_unpause_workflow_rs() {
    world().run("scenarios/pause_unpause_workflow.scen.json");
}

#[test]
fn rate_change_affects_future_rs() {
    world().run("scenarios/rate_change_affects_future.scen.json");
}

#[test]
fn unpause_non_owner_rs() {
    world().run("scenarios/unpause_non_owner.scen.json");
}

#[test]
fn unpause_not_paused_rs() {
    world().run("scenarios/unpause_not_paused.scen.json");
}

#[test]
fn unpause_success_rs() {
    world().run("scenarios/unpause_success.scen.json");
}

#[test]
fn withdraw_empty_rs() {
    world().run("scenarios/withdraw_empty.scen.json");
}

#[test]
fn withdraw_non_owner_rs() {
    world().run("scenarios/withdraw_non_owner.scen.json");
}

#[test]
fn withdraw_success_rs() {
    world().run("scenarios/withdraw_success.scen.json");
}

#[test]
fn migration_test_rs() {
    world().run("scenarios/migration_test.scen.json");
}
