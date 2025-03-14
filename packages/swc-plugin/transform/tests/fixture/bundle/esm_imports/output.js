import React, { useState, useCallback, useMemo as useMemoization } from 'react';
import Default from 'mod-1';
import { foo, bar, baz } from 'mod-2';
import * as all from 'mod-3';
const __deps = {
    "react": ()=>({
            default: React,
            useState,
            useCallback,
            useMemo: useMemoization
        }),
    "mod-3": ()=>({
            all
        }),
    "mod-1": ()=>({
            default: Default
        }),
    "mod-2": ()=>({
            foo,
            bar,
            baz
        })
};
global.__modules.define(function(__context) {
    const { default: React, useState, useCallback, useMemo: useMemoization } = __context.require("react");
    const { default: Default } = __context.require("mod-1");
    const { foo, bar, baz } = __context.require("mod-2");
    const { all } = __context.require("mod-3");
}, "1000", __deps);
