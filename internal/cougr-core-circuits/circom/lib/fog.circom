pragma circom 2.1.0;

include "comparators.circom";

// (tile_x - origin_x)^2 + (tile_y - origin_y)^2 <= radius^2
// Coordinates are non-negative map indices (0..2^bits).
template EuclideanDistanceLE(bits) {
    signal input origin_x;
    signal input origin_y;
    signal input tile_x;
    signal input tile_y;
    signal input visibility_radius;

    component ox = LessEqThan(bits);
    ox.in[0] <== origin_x;
    ox.in[1] <== (1 << bits) - 1;
    ox.out === 1;

    component oy = LessEqThan(bits);
    oy.in[0] <== origin_y;
    oy.in[1] <== (1 << bits) - 1;
    oy.out === 1;

    component tx = LessEqThan(bits);
    tx.in[0] <== tile_x;
    tx.in[1] <== (1 << bits) - 1;
    tx.out === 1;

    component ty = LessEqThan(bits);
    ty.in[0] <== tile_y;
    ty.in[1] <== (1 << bits) - 1;
    ty.out === 1;

    component rOk = LessEqThan(bits);
    rOk.in[0] <== visibility_radius;
    rOk.in[1] <== (1 << bits) - 1;
    rOk.out === 1;

    signal dx;
    signal dy;
    dx <== tile_x - origin_x;
    dy <== tile_y - origin_y;

    signal dx2;
    signal dy2;
    signal dist2;
    dx2 <== dx * dx;
    dy2 <== dy * dy;
    dist2 <== dx2 + dy2;

    signal radius2;
    radius2 <== visibility_radius * visibility_radius;

    component within = LessEqThan(64);
    within.in[0] <== dist2;
    within.in[1] <== radius2;
    within.out === 1;
}