# -*- coding: utf-8 -*-
import unittest
import numpy as np

from rascaline.systems import AseSystem

try:
    import ase

    HAVE_ASE = True
except ImportError:
    HAVE_ASE = False


@unittest.skipIf(not HAVE_ASE, "ASE not installed")
class TestAseSystem(unittest.TestCase):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        if not HAVE_ASE:
            return

        self.positions = [
            (0, 0, 0),
            (0, 0, 1.4),
            (0, 0, -1.6),
        ]
        atoms = ase.Atoms(
            "CO2",
            positions=self.positions,
        )
        atoms.pbc = [False, False, False]
        self.system = AseSystem(atoms)

    def test_system_implementation(self):
        self.assertEqual(self.system.size(), 3)
        self.assertTrue(np.all(self.system.species() == [6, 8, 8]))
        self.assertTrue(np.all(self.system.positions() == self.positions))
        self.assertTrue(np.all(self.system.cell() == [0, 0, 0, 0, 0, 0, 0, 0, 0]))

    def test_pairs(self):
        self.system.compute_neighbors(1.5)
        pairs = self.system.pairs()
        self.assertEqual(len(pairs), 1)
        self.assertEqual(pairs[0][:2], (0, 1))
        self.assertTrue(np.all(pairs[0][2] == [0, 0, 1.4]))

        self.system.compute_neighbors(2.5)
        pairs = self.system.pairs()
        self.assertEqual(len(pairs), 2)
        self.assertEqual(pairs[0][:2], (0, 1))
        self.assertTrue(np.all(pairs[0][2] == [0, 0, 1.4]))

        self.assertEqual(pairs[1][:2], (0, 2))
        self.assertTrue(np.all(pairs[1][2] == [0, 0, -1.6]))

        self.system.compute_neighbors(3.5)
        pairs = self.system.pairs()
        self.assertEqual(len(pairs), 3)
        self.assertEqual(pairs[0][:2], (0, 1))
        self.assertTrue(np.all(pairs[0][2] == [0, 0, 1.4]))

        self.assertEqual(pairs[1][:2], (0, 2))
        self.assertTrue(np.all(pairs[1][2] == [0, 0, -1.6]))

        self.assertEqual(pairs[2][:2], (1, 2))
        self.assertTrue(np.all(pairs[2][2] == [0, 0, -3.0]))

    def test_pairs_containing(self):
        self.system.compute_neighbors(1.5)
        pairs = self.system.pairs_containing(0)
        self.assertEqual(len(pairs), 1)
        self.assertEqual(pairs[0][:2], (0, 1))

        pairs = self.system.pairs_containing(1)
        self.assertEqual(len(pairs), 1)
        self.assertEqual(pairs[0][:2], (0, 1))

        pairs = self.system.pairs_containing(2)
        self.assertEqual(len(pairs), 0)

        self.system.compute_neighbors(3.5)
        pairs = self.system.pairs_containing(0)
        self.assertEqual(len(pairs), 2)
        self.assertEqual(pairs[0][:2], (0, 1))
        self.assertEqual(pairs[1][:2], (0, 2))

        pairs = self.system.pairs_containing(1)
        self.assertEqual(len(pairs), 2)
        self.assertEqual(pairs[0][:2], (0, 1))
        self.assertEqual(pairs[1][:2], (1, 2))

        pairs = self.system.pairs_containing(2)
        self.assertEqual(len(pairs), 2)
        self.assertEqual(pairs[0][:2], (0, 2))
        self.assertEqual(pairs[1][:2], (1, 2))
