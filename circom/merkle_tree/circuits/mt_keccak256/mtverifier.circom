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

/*

MTVerifier is a component to verify inclusion/exclusion of an element in the tree

 */
pragma circom 2.0.0;


include "../bitify.circom";
include "../comparators.circom";
include "../switcher.circom";
include "mtverifierlevel.circom";
include "mthash_keccak.circom";

template MTVerifier(nLevels) {
    signal input root[256];
    signal input siblings[nLevels-1][256];
    signal input child[256];
    signal input index;
    signal output out;

    component bits = Num2Bits(nLevels-1);
    bits.in <== index;

    component levels[nLevels-1];
    for (var i = 0; i < nLevels-1; i++) {
        levels[i] = MTVerifierLevel();

        for (var j = 0; j < 256; j++) {
            levels[i].sibling[j] <== siblings[i][j];
        }

        levels[i].lrbit <== bits.out[i];
        if (i == 0) {
            for (var j = 0; j < 256; j++) {
                levels[i].child[j] <== child[j];
            }
        } else {
            for (var j = 0; j < 256; j++) {
                levels[i].child[j] <== levels[i-1].root[j];
            }
        }
    }
    for (var i = 0; i < 256; i++) {
        levels[nLevels-2].root[i] === root[i];
    }
}

component main = MTVerifier(11);
