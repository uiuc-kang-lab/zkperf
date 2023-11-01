/*
    Copyright 2018 0KIMS association.

    This file is part of circom (Zero Knowledge Circuit Compiler).

    circom is a free software: you can redistribute it and/or modify it
    under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    circom is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
    or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public
    License for more details.

    You should have received a copy of the GNU General Public License
    along with circom. If not, see <https://www.gnu.org/licenses/>.
*/
pragma circom 2.0.0;

include "../keccak256/keccak.circom";

/*
    Hash1 = H(key | value)
 */

template SMTHash1() {
    signal input key[16];
    signal input value[496];
    signal output out[256];

    component h = Keccak(512, 256);
    for (var i = 0; i < 16; i++) {
        h.in[i] <== key[i];
    }
    for (var i = 0; i < 496; i++) {
        h.in[i+16] <== value[i];
    }

    for (var i = 0; i < 256; i++) {
        out[i] <== h.out[i];
    }
}

/*
    This component is used to create the 2 nodes.

    Hash2 = H(Hl | Hr)
 */

template SMTHash2() {
    signal input L[256];
    signal input R[256];
    signal output out[256];

    component h = Keccak(512, 256);
    for (var i = 0; i < 256; i++) {
        h.in[i] <== L[i];
    }
    for (var i = 0; i < 256; i++) {
        h.in[i+256] <== R[i];
    }

    for (var i = 0; i < 256; i++) {
        out[i] <== h.out[i];
    }
}
