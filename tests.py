#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Contains unit tests for yii-to-yii2.py
"""

import unittest
from yii_to_yii2 import *

class TestStringMethods(unittest.TestCase):
    def test_activequery(self):
        contents = [
            ("", ""),
            ("", "")
        ]
        for i in contents:
            result = activequery(i[0])
            self.assertEqual(result, i[1])
           
    def test_replaces(self):
        contents = [
            ("Yii::app()", "Yii::$app"),
            ("html/", "web/"),
            ("CActiveRecord", "ActiveRecord"),
            ("Yii::$app->createUrl('controller/page')", "Url::to('controller/page')"),
            ("$this->createUrl('controller/page')", "Url::to('controller/page')"),
            ("Html::link('controller/page')", "Html::a('controller/page')")
        ]
        replacements = [
            "yii-app",
            "html-web",
            "yii1-classes",
            "create-url",
            "static-table-name",
            "this-context",
            "render",
            "layout"
        ]
        with open("yii1-classes.txt", 'r') as file:
            yii1_classes = [i.strip() for i in file.readlines()]
        for i in contents:
            result = replace(i[0], yii1_classes, replacements)
            self.assertEqual(result, i[1])


if __name__ == '__main__':
    unittest.main()
