pragma circom 2.1.0;

include "poseidon.circom";
include "comparators.circom";
include "lib/fog.circom";

// Fog-of-war exploration: Euclidean visibility + Poseidon state transition bound to map_root.
//
// Public inputs match FogOfWarCircuit::verify_exploration in src/zk/advanced.rs.

template FogOfWar(MAX_RADIUS, COORD_BITS) {
    signal input map_root;
    signal input prior_explored_root;
    signal input next_explored_root;
    signal input origin_x;
    signal input origin_y;
    signal input tile_x;
    signal input tile_y;
    signal input visibility_radius;

    component radiusCap = LessEqThan(16);
    radiusCap.in[0] <== visibility_radius;
    radiusCap.in[1] <== MAX_RADIUS;
    radiusCap.out === 1;

    component dist = EuclideanDistanceLE(COORD_BITS);
    dist.origin_x <== origin_x;
    dist.origin_y <== origin_y;
    dist.tile_x <== tile_x;
    dist.tile_y <== tile_y;
    dist.visibility_radius <== visibility_radius;

    component transition = Poseidon(4);
    transition.inputs[0] <== map_root;
    transition.inputs[1] <== prior_explored_root;
    transition.inputs[2] <== tile_x;
    transition.inputs[3] <== tile_y;
    next_explored_root === transition.out;
}

component main {public [map_root, prior_explored_root, next_explored_root, origin_x, origin_y, tile_x, tile_y, visibility_radius]} = FogOfWar(8, 16);