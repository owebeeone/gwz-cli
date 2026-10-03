"""Provision the actual cargo-dist target matrix; normal Cargo stays unprovisioned."""
import argparse
import importlib.util
import hashlib
import json
import os
from pathlib import Path
import subprocess

def producer(manifest):
    metadata=json.loads(subprocess.check_output(['cargo','metadata','--locked','--format-version','1','--manifest-path',str(manifest)]))
    packages=[p for p in metadata['packages'] if p['name']=='gwz-sspi']
    if len(packages)!=1:raise RuntimeError('ambiguous or absent SSPI dependency')
    package=packages[0]
    path=Path(package['manifest_path']).parent/'scripts/artifact_set.py'
    spec=importlib.util.spec_from_file_location('artifact_set',path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--targets',help='cargo-dist matrix JSON')
    parser.add_argument('--dist-args',help='exact cargo-dist matrix options')
    parser.add_argument('--target',help='local build target triple')
    parser.add_argument('--profile',help='local Cargo profile (default release); matrix mode uses dist')
    parser.add_argument('--target-dir',type=Path,help='absolute external local build output')
    args=parser.parse_args();manifest=Path(__file__).resolve().parents[1]/'Cargo.toml';module=producer(manifest)
    if args.targets:
        targets=json.loads(args.targets);profile='dist';options={'dist_args':args.dist_args}
        if args.dist_args is None or args.target or args.target_dir or args.profile:parser.error('dist mode requires matrix options only')
    else:
        if args.dist_args is not None:parser.error('local mode does not accept dist options')
        if not args.target or not args.target_dir or not args.target_dir.is_absolute():parser.error('local mode requires --target and absolute --target-dir')
        targets=[args.target];profile=args.profile or 'release';options={'command':'cargo build'}
    if not isinstance(targets,list) or not targets or not all(isinstance(t,str) and t and t.isascii() and all(c.isalnum() or c in '-_' for c in t) for t in targets) or len(set(targets))!=len(targets):raise SystemExit('invalid targets')
    table=[];receipts=[]
    for target in targets:
        fingerprint,_,inputs=module.identify(manifest,target=target,profile=profile,options=options)
        table.append(target+'='+fingerprint);receipts.append(json.loads(module.receipt(fingerprint,inputs)))
    value='\n'.join(table)
    if args.targets:
        with Path(os.environ['GITHUB_ENV']).open('a') as output:output.write('GWZ_SSPI_PACKAGING_TARGETS<<GWZ_SSPI_END\n'+value+'\nGWZ_SSPI_END\n')
        destination=Path(os.environ.get('CARGO_TARGET_DIR','target'))/'distrib'
    else:
        environment={**os.environ,'GWZ_SSPI_PACKAGING_TARGETS':value,'CARGO_TARGET_DIR':str(args.target_dir)}
        subprocess.run(['cargo','build','--locked','--manifest-path',str(manifest),'--target',args.target,'--profile',profile],env=environment,check=True)
        destination=args.target_dir/'distrib'
    destination.mkdir(parents=True,exist_ok=True)
    receipt_name='sspi-artifact-sets-'+hashlib.sha256('\0'.join(sorted(targets)).encode()).hexdigest()+'.json' if args.targets else 'sspi-artifact-sets.json'
    (destination/receipt_name).write_text(json.dumps(receipts,sort_keys=True,indent=2)+'\n')
if __name__=='__main__':main()
