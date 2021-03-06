#include <cstring>
#include <cassert>

#include <stdexcept>

#include "helpers.hpp"

rascal_system_t simple_system() {
    rascal_system_t system;
    std::memset(&system, 0, sizeof(system));

    system.size = [](const void* _, uintptr_t* size){
        *size = 4;
    };

    system.positions = [](const void* _, const double** positions){
        static double POSITIONS[4][3] = {
            {0, 0, 0},
            {1, 1, 1},
            {2, 2, 2},
            {3, 3, 3},
        };
        *positions = POSITIONS[0];
    };

    system.species = [](const void* _, const uintptr_t** species){
        static uintptr_t SPECIES[4] = {6, 1, 1, 1};
        *species = SPECIES;
    };

    system.cell = [](const void* _, double* cell){
        static double CELL[3][3] = {
            {10, 0, 0},
            {0, 10, 0},
            {0, 0, 10},
        };
        std::memcpy(cell, CELL, sizeof(CELL));
    };

    // basic compute_neighbors, always returning the same pairs
    system.compute_neighbors = [](void* _, double cutoff){
        assert(cutoff > 1.73205080756887729352 && cutoff < 3.46410161513775458704);
    };

    system.pairs = [](const void* _, const rascal_pair_t** pairs, uintptr_t* count){
        static rascal_pair_t PAIRS[] = {
            {0, 1, {1, 1, 1}},
            {1, 2, {1, 1, 1}},
            {2, 3, {1, 1, 1}},
        };

        *pairs = PAIRS;
        *count = 3;
    };

    system.pairs_containing = [](const void* _, uintptr_t center, const rascal_pair_t** pairs, uintptr_t* count){
        static rascal_pair_t PAIRS_0[] = {
            {0, 1, {1, 1, 1}},
        };

        static rascal_pair_t PAIRS_1[] = {
            {0, 1, {1, 1, 1}},
            {1, 2, {1, 1, 1}},
        };

        static rascal_pair_t PAIRS_2[] = {
            {1, 2, {1, 1, 1}},
            {2, 3, {1, 1, 1}},
        };

        static rascal_pair_t PAIRS_3[] = {
            {2, 3, {1, 1, 1}},
        };

        if (center == 0) {
            *pairs = PAIRS_0;
            *count = 1;
        } else if (center == 1) {
            *pairs = PAIRS_1;
            *count = 2;
        } else if (center == 2) {
            *pairs = PAIRS_2;
            *count = 2;
        } else if (center == 3) {
            *pairs = PAIRS_3;
            *count = 1;
        } else {
            throw std::runtime_error("got invalid center atom");
        }
    };

    return system;
}
