go build -o gmt
mkdir msm_data
mv gmt msm_data/gmt
cd msm_data
./gmt msm 1000
./gmt msm 10000
mv gmt ../gmt
cd ..

mkdir fft_data
mv gmt fft_data/gmt
cd fft_data
./gmt fft 1024
./gmt fft 8192
mv gmt ../gmt
cd ..

mkdir circuit_data
mv gmt circuit_data/gmt
cd circuit_data
./gmt circuit 1000
./gmt circuit 10000
mv gmt ../gmt
cd ..