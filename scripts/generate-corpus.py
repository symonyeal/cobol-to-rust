#!/usr/bin/env python3
"""Generate deterministic input scenarios; outputs are not expected-result bytes."""
import argparse
import json
import random
from pathlib import Path

SEED = 20261106
OUT = Path(__file__).resolve().parents[1] / "docs" / "source-corpora.json"


def hx(data: bytes) -> str:
    return data.hex()


def fx_record(name: str, price: str, filler: bytes = b" " * 17) -> bytes:
    name_bytes = name.encode("ascii").ljust(50, b" ")
    price_bytes = price.encode("ascii").rjust(13, b" ")
    record = name_bytes + price_bytes + filler
    if len(name_bytes) != 50 or len(price_bytes) != 13 or len(record) != 80:
        raise ValueError("fxsort record must be 80 bytes")
    return record


def fx_files(boss=(), mxr=(), jhs=()) -> dict[str, bytes]:
    return {
        "FXBOSS": b"".join(boss),
        "FXMXR": b"".join(mxr),
        "FXJHS": b"".join(jhs),
    }


def scenario(case: str, scenario_id: str, domain: str, files: dict[str, bytes]) -> dict:
    return {
        "case": case,
        "id": scenario_id,
        "domain": domain,
        "seed": SEED,
        "initial_hex": {name: hx(content) for name, content in sorted(files.items())},
        "port_args": [],
    }


def fxsort_scenarios() -> list[dict]:
    result = [scenario("fxsort", "empty-inputs", "all three inputs empty", fx_files())]
    result.append(scenario(
        "fxsort", "single-record", "one 80-byte record in one input",
        fx_files(boss=[fx_record("SINGLE", "$1.00")]),
    ))
    result.append(scenario(
        "fxsort", "duplicate-keys-and-ties", "duplicate names and equal prices across input files",
        fx_files(
            boss=[fx_record("ALPHA", "$12.34"), fx_record("DUPLICATE", "$2.00")],
            mxr=[fx_record("ALPHA", "$12.34"), fx_record("DUPLICATE", "$2.00")],
            jhs=[fx_record("BETA", "$12.34")],
        ),
    ))
    result.append(scenario(
        "fxsort", "leading-zero-and-small-values", "leading-zero names and minimum cent values",
        fx_files(
            boss=[fx_record("00000000000000000001", "$0.01")],
            mxr=[fx_record("00000000000000000002", "$0.00")],
        ),
    ))
    result.append(scenario("fxsort", "character-key-order", "raw character price keys whose byte order differs from decimal interpretation",
                           fx_files(boss=[fx_record("A", "$000000002.00")], mxr=[fx_record("B", "$12.00")])))
    result.append(scenario(
        "fxsort", "currency-boundaries", "zero, cent, and maximum 8-digit integer currency",
        fx_files(
            boss=[fx_record("MAXIMUM", "99,999,999.99")],
            jhs=[fx_record("ZERO", "$0.00"), fx_record("CENT", "$0.01")],
        ),
    ))
    complete = fx_record("COMPLETE", "$1.00", b"A" * 17)
    partial = fx_record("PARTIAL", "$2.00", b"B" * 17)[:-1]
    result.append(scenario(
        "fxsort", "partial-fixed-record", "one 80-byte record followed by a 79-byte partial record",
        fx_files(boss=[complete, partial]),
    ))
    result.append(scenario("fxsort", "one-byte-record", "one-byte fixed record; verify exact space padding without stale data",
                           fx_files(boss=[b"A"])))
    result.append(scenario(
        "fxsort", "missing-input-file", "required FXJHS input file absent",
        {"FXBOSS": fx_record("PRESENT", "$1.00"), "FXMXR": b""},
    ))
    rng = random.Random(SEED)
    groups = []
    for group in range(3):
        records = [fx_record(rng.choice(["ALPHA", "alpha", "!BETA", "0001", "Z"]),
                             rng.choice(["$2.00", "$12.00", "$0.00", "$999.99"]),
                             f"source{group}-row{i:03d}".encode().ljust(17, b" ")) for i in range(50)]
        groups.append(sorted(records, key=lambda record: record[:50]))
    result.append(scenario("fxsort", "generated-three-way-merge",
                           "seed 20261106: 150 records; each input sorted by raw name; ties, punctuation, case and distinguishable payloads",
                           fx_files(*groups)))
    return result


def sales_record(index: int, *, product: str = "001", quantity: str = "001", cost: str = "00100") -> bytes:
    customer_id = f"{index % 100000:05d}"
    customer_name = f"CLIENT {index:05d}".encode("ascii").ljust(25, b" ")
    product_name = f"ITEM {index % 10000:04d}".encode("ascii").ljust(14, b" ")
    record = (
        customer_id.encode("ascii") + customer_name + product.encode("ascii")
        + b" " * 5 + product_name + quantity.encode("ascii") + cost.encode("ascii")
    )
    if len(record) != 60:
        raise ValueError("sales input record must be 60 bytes")
    return record


def sales_file(records: list[bytes]) -> bytes:
    return b"\n".join(records) + (b"\n" if records else b"")


def sales_scenarios() -> list[dict]:
    case = "sales-report"
    result = [scenario(case, "empty-input", "empty line-sequential input", {"PR2FA17.txt": b""})]
    result.append(scenario(case, "single-record", "one 60-byte sales record", {"PR2FA17.txt": sales_file([sales_record(1)])}))
    result.append(scenario(
        case, "duplicate-product-group", "consecutive equal product IDs and repeated customers",
        {"PR2FA17.txt": sales_file([
            sales_record(1, product="007", quantity="002", cost="00050"),
            sales_record(1, product="007", quantity="003", cost="00075"),
            sales_record(2, product="008", quantity="001", cost="00100"),
        ])},
    ))
    result.append(scenario(
        case, "leading-zero-values", "zero-padded identifiers, quantity, and cost",
        {"PR2FA17.txt": sales_file([sales_record(1, product="001", quantity="007", cost="00005")])},
    ))
    result.append(scenario(
        case, "numeric-boundaries", "zero and maximum declared quantity/cost fields",
        {"PR2FA17.txt": sales_file([
            sales_record(1, quantity="000", cost="00000"),
            sales_record(2, quantity="999", cost="99999"),
        ])},
    ))
    result.append(scenario(
        case, "partial-line-record", "59-byte line where the declared input record is 60 bytes",
        {"PR2FA17.txt": sales_record(1)[:-1] + b"\n"},
    ))
    result.append(scenario(case, "missing-input-file", "PR2FA17.txt absent", {}))
    for count in (44, 45, 46):
        records = [sales_record(i + 1, product="001" if i < count // 2 else "002") for i in range(count)]
        result.append(scenario(
            case, f"page-{count}-records", f"{count} records around the source page-break threshold",
            {"PR2FA17.txt": sales_file(records)},
        ))
    result.append(scenario(case, "total-value-truncation", "11 maximum-value records in one group overflow the 9-digit cents total destination",
                           {"PR2FA17.txt": sales_file([sales_record(i, quantity="999", cost="99999") for i in range(11)])}))
    result.append(scenario(case, "total-quantity-truncation", "1003 records of quantity 999 overflow the 6-digit group quantity destination; repeated pages",
                           {"PR2FA17.txt": sales_file([sales_record(i, quantity="999", cost="00001") for i in range(1003)])}))
    result.append(scenario(case, "edited-quantity-zeroes", "group quantity 100001 retains internal zeroes in PIC ZZZ999 and final comma editing",
                           {"PR2FA17.txt": sales_file([sales_record(i,quantity="999",cost="00000") for i in range(100)] + [sales_record(100,quantity="101",cost="00000")])}))
    rng = random.Random(SEED)
    records = [sales_record(i, product=f"{rng.randrange(6):03d}", quantity=f"{rng.randrange(1000):03d}", cost=f"{rng.randrange(100000):05d}") for i in range(200)]
    result.append(scenario(case, "generated-group-transitions", "seed 20261106: 200 records with repeated and singleton groups, zero-padded identifiers, bounded decimal fields and page transitions",
                           {"PR2FA17.txt": sales_file(records)}))
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail if generated corpus or case fixtures differ")
    args = parser.parse_args()
    scenarios = fxsort_scenarios() + sales_scenarios()
    payload = json.dumps(scenarios, indent=2, ensure_ascii=True) + "\n"
    if args.check:
        if OUT.read_text(encoding="utf-8") != payload:
            raise ValueError("generated corpus is stale")
    else:
        OUT.write_text(payload, encoding="utf-8", newline="\n")
    for case_id, source_id in (("fxsort", "fxsort"), ("sales", "sales-report")):
        manifest = OUT.parents[1] / "cases" / f"{case_id}.json"
        case = json.loads(manifest.read_text(encoding="utf-8"))
        generated = [{k:s[k] for k in ("id", "domain", "initial_hex", "port_args")} for s in scenarios if s["case"] == source_id]
        if args.check:
            if case["scenarios"] != generated:
                raise ValueError(f"{case_id} case fixtures differ from generator")
        else:
            case["scenarios"] = generated
            manifest.write_text(json.dumps(case, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(f"{'checked' if args.check else 'wrote'} {len(scenarios)} deterministic scenarios; seed={SEED}")


if __name__ == "__main__":
    main()
