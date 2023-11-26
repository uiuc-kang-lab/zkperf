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
./gmt circuit 8388608
mv gmt ../gmt
cd ..