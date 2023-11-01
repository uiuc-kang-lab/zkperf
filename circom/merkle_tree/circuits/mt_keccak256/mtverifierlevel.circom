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

/******

SMTVerifierLevel

This circuit has 1 hash

Outputs according to the state.

State        root
=====        =======
top          H'(child, sibling)
i0           0
iold         old1leaf
inew         new1leaf
na           0

H' is the Hash function with the inputs shifted acordingly.

*****/
pragma circom 2.0.0;

template MTVerifierLevel() {

    signal input lrbit;
    signal output root[256];
    signal input child[256];
    signal input sibling[256];

    component proofHash = SMTHash2();
    component switcher[256];

    for (var i = 0; i < 256; i++) {
        switcher[i] =  Switcher();
        switcher[i].L <== child[i];
        switcher[i].R <== sibling[i];
        switcher[i].sel <== lrbit;
    }
    for (var i = 0; i < 256; i++) {
        proofHash.L[i] <== switcher[i].outL;
        proofHash.R[i] <== switcher[i].outR;
    }
    for (var i = 0; i < 256; i++) {
        root[i] <== proofHash.out[i];
    }

}
