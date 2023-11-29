go build -o gmt
mkdir msm_data
mv gmt msm_data/gmt
cd msm_data
./gmt msm 1024
./gmt msm 2048
./gmt msm 4096
./gmt msm 8192
./gmt msm 16384
./gmt msm 32768
./gmt msm 65536
./gmt msm 131072
./gmt msm 262144
./gmt msm 524288
./gmt msm 1048576
./gmt msm 2097152
./gmt msm 4194304
./gmt msm 8388608
mv gmt ../gmt
cd ..

mkdir fft_data
mv gmt fft_data/gmt
cd fft_data
./gmt fft 1024
./gmt fft 2048
./gmt fft 4096
./gmt fft 8192
./gmt fft 16384
./gmt fft 32768
./gmt fft 65536
./gmt fft 131072
./gmt fft 262144
./gmt fft 524288
./gmt fft 1048576
./gmt fft 2097152
./gmt fft 4194304
./gmt fft 8388608
mv gmt ../gmt
cd ..

mkdir circuit_data
mv gmt circuit_data/gmt
cd circuit_data
./gmt circuit 1024
./gmt circuit 2048
./gmt circuit 4096
./gmt circuit 8192
./gmt circuit 16384
./gmt circuit 32768
./gmt circuit 65536
./gmt circuit 131072
./gmt circuit 262144
./gmt circuit 524288
./gmt circuit 1048576
./gmt circuit 2097152
./gmt circuit 4194304
mv gmt ../gmt
cd ..

mkdir arithmetic_data
mv gmt arithmetic_data/gmt
cd arithmetic_data
./gmt arithmetic 65536
./gmt arithmetic 131072
./gmt arithmetic 262144
./gmt arithmetic 524288
./gmt arithmetic 1048576
./gmt arithmetic 2097152
./gmt arithmetic 4194304
./gmt arithmetic 8388608
./gmt arithmetic 16777216
./gmt arithmetic 33554432
mv gmt ../gmt
cd ..

# relu script

mkdir relu_data
./gmt relu 1024 relu_data/
mv *.json relu_breakdown_1024.json
mv relu_breakdown_1024.json relu_data/relu_breakdown_1024.json
./gmt relu 2048
mv *.json relu_breakdown_2048.json
mv relu_breakdown_2048.json relu_data/relu_breakdown_2048.json
./gmt relu 4096
mv .json relu_breakdown_4096.json
mv relu_breakdown_4096.json relu_data/relu_breakdown_4096.json
./gmt relu 8192
mv .json relu_breakdown_8192.json
mv relu_breakdown_8192.json relu_data/relu_breakdown_8192.json
./gmt relu 16384
mv .json relu_breakdown_16384.json
mv relu_breakdown_16384.json relu_data/relu_breakdown_16384.json
./gmt relu 32768
mv .json relu_breakdown_32768.json
mv relu_breakdown_32768.json relu_data/relu_breakdown_32768.json
./gmt relu 65536
mv .json relu_breakdown_65536.json
mv relu_breakdown_65536.json relu_data/relu_breakdown_65536.json
./gmt relu 131072
mv .json relu_breakdown_131072.json
mv relu_breakdown_131072.json relu_data/relu_breakdown_131072.json
./gmt relu 262144
mv .json relu_breakdown_262144.json
mv relu_breakdown_262144.json relu_data/relu_breakdown_262144.json
./gmt relu 524288
mv .json relu_breakdown_524288.json
mv relu_breakdown_524288.json relu_data/relu_breakdown_524288.json


mkdir relu6_data
./gmt relu6 1024
mv *.json relu6_breakdown_1024.json
mv relu6_breakdown_1024.json relu6_data/relu6_breakdown_1024.json
./gmt relu6 2048
mv *.json relu6_breakdown_2048.json
mv relu6_breakdown_2048.json relu6_data/relu6_breakdown_2048.json
./gmt relu6 4096
mv .json relu6_breakdown_4096.json
mv relu6_breakdown_4096.json relu6_data/relu6_breakdown_4096.json
./gmt relu6 8192
mv .json relu6_breakdown_8192.json
mv relu6_breakdown_8192.json relu6_data/relu6_breakdown_8192.json
./gmt relu6 16384
mv .json relu6_breakdown_16384.json
mv relu6_breakdown_16384.json relu6_data/relu6_breakdown_16384.json
./gmt relu6 32768
mv .json relu6_breakdown_32768.json
mv relu6_breakdown_32768.json relu6_data/relu6_breakdown_32768.json
./gmt relu6 65536
mv .json relu6_breakdown_65536.json
mv relu6_breakdown_65536.json relu6_data/relu6_breakdown_65536.json
./gmt relu6 131072
mv .json relu6_breakdown_131072.json
mv relu6_breakdown_131072.json relu6_data/relu6_breakdown_131072.json
./gmt relu6 262144
mv .json relu6_breakdown_262144.json
mv relu6_breakdown_262144.json relu6_data/relu6_breakdown_262144.json
./gmt relu6 524288
mv .json relu6_breakdown_524288.json
mv relu6_breakdown_524288.json relu6_data/relu6_breakdown_524288.json
