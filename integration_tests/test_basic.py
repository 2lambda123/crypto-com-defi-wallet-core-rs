#!/usr/bin/env python3


def test_basic(chainmain, cronos):
    singer1_addr = chainmain.cosmos_cli(0).address("signer1")
    singer2_addr = chainmain.cosmos_cli(0).address("signer2")
    community_addr = chainmain.cosmos_cli(0).address("community")
    delegator_addr = chainmain.cosmos_cli(0).address("delegator")
    validator_addr = chainmain.cosmos_cli(0).address("validator")
    print(singer1_addr, singer2_addr, community_addr, delegator_addr, validator_addr)

    rpc = chainmain.node_rpc(0)
    print(rpc)

    w3 = cronos.w3
    print(w3.eth.get_block_number())
