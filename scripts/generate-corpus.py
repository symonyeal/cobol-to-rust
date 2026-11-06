#!/usr/bin/env python3
"""Generate deterministic input scenarios; outputs are not expected-result bytes."""
import json
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
    result.append(scenario(
        "fxsort", "currency-boundaries", "zero, cent, and maximum 8-digit integer currency",
        fx_files(
            boss=[fx_record("MAXIMUM", "99,999,999.99")],
            jhs=[fx_record("ZERO", "$0.00"), fx_record("CENT", "$0.01")],
        ),
    ))
    complete = fx_record("COMPLETE", "$1.00")
    result.append(scenario(
        "fxsort", "partial-fixed-record", "one 80-byte record followed by a 79-byte partial record",
        fx_files(boss=[complete, complete[:-1]]),
    ))
    result.append(scenario(
        "fxsort", "missing-input-file", "required FXJHS input file absent",
        {"FXBOSS": b"", "FXMXR": b""},
    ))
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
    return result


def main() -> None:
    scenarios = fxsort_scenarios() + sales_scenarios()
    OUT.write_text(json.dumps(scenarios, indent=2, ensure_ascii=True) + "\n", encoding="utf-8")
    print(f"wrote {len(scenarios)} deterministic scenarios to {OUT.name}; seed={SEED}")


if __name__ == "__main__":
    main()
