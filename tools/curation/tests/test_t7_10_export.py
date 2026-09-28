from sc_curation import export_public


def test_t7_10_parses_icp_candid_text():
    text = ('(opt record { v = 1 : nat8; subject_id = 10_000_086 : nat32; '
            'consensus = vec { record { "shape"; "smooth" }; record { "merger"; "none" } }; '
            'resolved_at = 1_790_000_000 : nat64 })')
    assert export_public.parse_candid(text) == {
        "v": 1, "subject_id": 10000086,
        "consensus": [["shape", "smooth"], ["merger", "none"]], "resolved_at": 1790000000,
    }
    assert export_public.parse_candid("(null)") is None
    assert export_public.parse_candid("(record { next_cursor = opt (188 : nat64); items = vec { 1 : nat8 } })") == {
        "next_cursor": 188, "items": [1]}
    assert export_public.parse_candid('(record { next_cursor = null; items = vec {} })') == {"next_cursor": None, "items": []}
    assert export_public.parse_candid('(variant { Confirmed }, -1.5 : float64, true, principal "aaaaa-aa")') == [
        "Confirmed", -1.5, True, "aaaaa-aa"]
    assert export_public.parse_candid('(variant { Ok = "a \\"q\\"" })') == {"Ok": 'a "q"'}


def fake_caller(responses):
    def caller(method, args, network="local", identity="anonymous"):
        return responses[(method, args)]
    return caller


def test_t7_10_consensus_export_skips_unresolved_and_computes_votes_ignoring_mismatches():
    cons = {"v": 1, "subject_id": 1, "consensus": [["shape", "smooth"]], "resolved_at": 9}
    votes = [
        {"answers": [{"question_id": "shape", "answer_id": "smooth"}], "image_mismatch": False},
        {"answers": [{"question_id": "shape", "answer_id": "featured"}], "image_mismatch": False},
        {"answers": [{"question_id": "shape", "answer_id": "featured"}], "image_mismatch": True},
    ]
    caller = fake_caller({
        ("get_subject_consensus", "(1 : nat32)"): cons,
        ("get_subject_consensus", "(2 : nat32)"): None,
        ("list_subject_classifications", "(1 : nat32)"): votes,
    })
    rows = export_public.fetch_consensus([2, 1], caller)
    assert rows == [{**cons, "votes": {"shape": {"smooth": 1, "featured": 1}}}]


def test_t7_10_consensus_export_without_admin_keeps_consensus_only():
    cons = {"v": 1, "subject_id": 1, "consensus": [["shape", "smooth"]], "resolved_at": 9}
    caller = fake_caller({
        ("get_subject_consensus", "(1 : nat32)"): cons,
        ("list_subject_classifications", "(1 : nat32)"): [],
    })
    assert export_public.fetch_consensus([1], caller) == [cons]
