import { useEffect } from 'react';
import { Counter } from './Counter';

function App() {
  useEffect(() => {
    console.log('App mounted');
  }, []);

  console.log('App rendered !!');

  return (
    <div className="App">
      <header className="App-header">
        <p>Counter state should be preserved across HMRs.</p>
        <Counter />
        <p>
          Edit<code>src/*</code> and save to reload.
          <br />
        </p>
        <p>
          Powered by{' '}
          <a
            className="App-link"
            href="https://github.com/leegeunhyeok/global-modules"
            target="_blank"
            rel="noopener noreferrer"
          >
            @global-modules
          </a>
        </p>
      </header>
    </div>
  );
}

App.__tag = new Date().toUTCString();

console.log('App loaded @@ 5');

export default App;
