import foo from 'foo';
const __deps = [
    ()=>({
            default: foo
        }),
    ()=>require('foo'),
    ()=>import('foo')
];
global.__modules.define(function(__context) {
    const { default: foo } = __context.require("foo", 0);
    __context.require("foo", 1);
    __context.import("foo", 2);
}, "1000", __deps);
