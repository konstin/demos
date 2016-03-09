#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Contains unit tests for yii-to-yii2.py
"""

import unittest
from yii-to-yii2 import *

class TestStringMethods(unittest.TestCase):
    def test_activequery(self):
        contents = [
            ("", ""),
            ("", "")
        ]
        for i in contents:
            result = activequery(i[0])
            self.assertEqual(result, i[1])


if __name__ == '__main__':
    unittest.main()
