pragma circom 2.0.0;

template Dummy(n) {
    signal input a[n];
    signal input out;
    signal output placeholder;
    
    signal buf[n+1];

    buf[0] <-- 1;

    for (var i = 0; i < n; i++) {
        buf[i+1] <== a[i] * buf[0];
    }
    out === buf[n];
}
