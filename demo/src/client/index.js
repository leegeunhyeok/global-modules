import { calculator } from './calculator';
import { calculator as negativeCalculator } from './negative-calculator';

window.addEventListener('load', () => {
  renderResult(calculate());
  updateBannerColor();
});

// Handler for lazy evaluation (eg. HMR update)
if (document.readyState === 'complete') {
  renderResult(calculate());
  updateBannerColor();
}

function calculate() {
  const result = {
    add: calculator.add(10, 5),
    sub: calculator.sub(10, 5),
    mul: calculator.mul(10, 5),
    div: calculator.div(10, 5),
    negativeAdd: negativeCalculator.add(10, 5),
    negativeSub: negativeCalculator.sub(10, 5),
    negativeMul: negativeCalculator.mul(10, 5),
    negativeDiv: negativeCalculator.div(10, 5),
  };

  return result;
}

function updateBannerColor() {
  const banner = document.getElementById('banner');

  if (banner) {
    banner.style.backgroundColor = `#${Math.floor(Math.random() * 16777215).toString(16)}`;
  }
}

function renderResult(results) {
  console.log('Render results:', results);
  const resultElement = document.getElementById('result');

  if (resultElement) {
    resultElement.textContent = JSON.stringify(results, null, 2);
  }
}
