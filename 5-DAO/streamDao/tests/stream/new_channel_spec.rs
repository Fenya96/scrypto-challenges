use radix_engine::ledger::*;
use scrypto::core::NetworkDefinition;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_new_channel() {
    let mut store: TypedInMemorySubstateStore = TypedInMemorySubstateStore::with_bootstrap();

    let mut test_runner: TestRunner<TypedInMemorySubstateStore> = TestRunner::new(true, &mut store);

    let (public_key, _private_key, account_component) = test_runner.new_account();

    let package_address = test_runner.compile_and_publish(this_package!());

    let subscription_price: Decimal = dec!("20");
    let create_channel_price: Decimal = dec!("50");
    let amount_rewards_subscription: Decimal = dec!("15");
    let amount_rewards_creating_channel: Decimal = dec!("15");
    let platform_fee: Decimal = dec!("5");

    let instantiate_manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .call_method(account_component, "lock_fee", args!(dec!("10")))
        .call_function(
            package_address,
            "Streamdao",
            "instantiate_streamdao",
            args!(
                subscription_price,
                create_channel_price,
                amount_rewards_subscription,
                amount_rewards_creating_channel,
                platform_fee
            ),
        )
        .call_method(
            account_component,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();

    let instantiation_receipt =
        test_runner.execute_manifest(instantiate_manifest, vec![public_key.into()]);

    let component = instantiation_receipt
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    let mint_member_address = instantiation_receipt
        .expect_commit()
        .entity_changes
        .new_resource_addresses[1];

    let membership_id_1: u64 = 95u64;

    let new_user_manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .call_method(account_component, "lock_fee", args!(dec!("10")))
        .call_method(
            component,
            "new_membership_set_id",
            args!(membership_id_1, "Dan".to_string()),
        )
        .call_method(
            account_component,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();

    test_runner.execute_manifest(new_user_manifest, vec![public_key.into()]);

    let mut membership_bt_1: BTreeSet<NonFungibleId> = BTreeSet::new();

    membership_bt_1.insert(NonFungibleId::from_u64(membership_id_1));

    let new_channel_manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .call_method(account_component, "lock_fee", args!(dec!("10")))
        .withdraw_from_account_by_amount(dec!("100"), RADIX_TOKEN, account_component)
        .take_from_worktop_by_amount(dec!("100"), RADIX_TOKEN, |build, xrd_bucket_id| {
            build
                .withdraw_from_account_by_ids(
                    &membership_bt_1,
                    mint_member_address,
                    account_component,
                )
                .take_from_worktop_by_ids(
                    &membership_bt_1,
                    mint_member_address,
                    |build, user_nft_bucket_id| {
                        build.call_method(
                            component,
                            "new_channel",
                            args!(
                                Bucket(xrd_bucket_id),
                                Bucket(user_nft_bucket_id),
                                "fuserleer".to_string()
                            ),
                        )
                    },
                )
        })
        .call_method(
            account_component,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();

    let recept = test_runner.execute_manifest(new_channel_manifest, vec![public_key.into()]);

    println!("{:?\n}", recept);
    recept.expect_commit_success();
}
