import { calculator } from './calculator';
import { calculator as negativeCalculator } from './negative-calculator';

console.group('calculator');
console.log('add', calculator.add(10, 5));
console.log('sub', calculator.sub(10, 5));
console.log('mul', calculator.mul(10, 5));
console.log('div', calculator.div(10, 5));
console.groupEnd();

console.group('calculator-negative');
console.log('add', negativeCalculator.add(10, 5));
console.log('sub', negativeCalculator.sub(10, 5));
console.log('mul', negativeCalculator.mul(10, 5));
console.log('div', negativeCalculator.div(10, 5));
console.groupEnd();
