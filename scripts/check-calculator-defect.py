"""Inject a subtraction defect, require comparator rejection, and restore source."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for flag in ('profile', 'sources', 'runs', 'label'):
        parser.add_argument('--'+flag, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    source = root / 'src/calculator.rs'
    original = source.read_bytes()
    needle = b"b'-' => a.abs_diff(b),"
    assert original.count(needle) == 1
    baseline = Path(args.runs).resolve() / 'calculator' / f'baseline-{args.label}.json'
    digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    before = digest(baseline)
    common = [value for key in ('profile','sources','runs','label') for value in ('--'+key,getattr(args,key))]
    run_dir = baseline.parent
    previous = set(run_dir.glob(f'verification-{args.label}-*.json'))
    try:
        source.write_bytes(original.replace(needle, b"b'-' => a.saturating_sub(b),"))
        subprocess.run(['cargo','build','--locked'],cwd=root,check=True)
        failure = subprocess.run([str(root/'target/debug/c2r.exe'),'verify','calculator',*common],cwd=root,capture_output=True,check=False)
        assert failure.returncode == 1, failure.stderr
        fresh = set(run_dir.glob(f'verification-{args.label}-*.json')) - previous
        assert len(fresh) == 1
        report = json.loads(fresh.pop().read_text())
        rejected = [s for s in report['scenarios'] if not s['comparison']['equal']]
        assert [s['scenario'] for s in rejected] == ['negative-result']
        assert report['status'] == 'mismatch' and digest(baseline) == before
        record = dict(schema_version=1,mutation='a.abs_diff(b) -> a.saturating_sub(b)',exit=failure.returncode,
                      baseline_sha256=before,baseline_unchanged=True,rejected=[dict(scenario=s['scenario'],comparison=s['comparison']) for s in rejected])
        output = Path(args.runs).resolve() / f'calculator-defect-{args.label}.json'
        with output.open('x',encoding='utf-8',newline='\n') as handle:
            handle.write(json.dumps(record,indent=2)+'\n')
        print(json.dumps(record,indent=2))
    finally:
        source.write_bytes(original)
        subprocess.run(['cargo','build','--locked'],cwd=root,check=True)
        assert source.read_bytes() == original
    subprocess.run([str(root/'target/debug/c2r.exe'),'verify','calculator',*common],cwd=root,check=True)


if __name__ == '__main__':
    main()
