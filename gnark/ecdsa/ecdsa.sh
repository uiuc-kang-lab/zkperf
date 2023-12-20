go test -v -run TestSetup -timeout 0
{ /usr/bin/time -v go test -v -run TestECDSAGroth16BN254 -timeout 0; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "ecdsa_groth16.json")" > "ecdsa_groth16.json"
go test -v -run TestECDSAGroth16Verifier -timeout 0
{ /usr/bin/time -v go test -v -run TestECDSAPlonkBN254 -timeout 0; } 2> /tmp/test
echo "$(jq --arg tmp $(echo "scale=6; $(cat /tmp/test | grep "Maximum resident set size" | tr -d -c 0-9)/1024" | bc) '.+={"MemoryConsumption": $tmp }' "ecdsa_plonk.json")" > "ecdsa_plonk.json"
go test -v -run TestECDSAPlonkVerifier -timeout 0
