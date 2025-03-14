import React, { useState, useCallback, useMemo as useMemoization } from 'react';
import Default from 'mod-1';
import { foo, bar, baz } from 'mod-2';
import * as all from 'mod-3';
const __deps = [
    ()=>({
            default: React,
            useState,
            useCallback,
            useMemo: useMemoization
        }),
    ()=>({
            default: Default
        }),
    ()=>({
            foo,
            bar,
            baz
        }),
    ()=>({
            all
        })
];
global.__modules.define(function(__context) {
    const { default: React, useState, useCallback, useMemo: useMemoization } = __context.require("react", 0);
    const { default: Default } = __context.require("mod-1", 1);
    const { foo, bar, baz } = __context.require("mod-2", 2);
    const { all } = __context.require("mod-3", 3);
}, "1000", __deps);
