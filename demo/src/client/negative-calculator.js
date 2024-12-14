import { add } from './add';
import { sub } from './sub';
import { mul } from './mul';
import { div } from './div';

export const calculator = {
  add: (a, b) => -add(a, b),
  sub: (a, b) => -sub(a, b),
  mul: (a, b) => -mul(a, b),
  div: (a, b) => -div(a, b),
};
