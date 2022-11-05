import React from 'react';
import './App.css';
import Choice from './components/Choice/choice'
import Question from './components/Question/question'

function App() {
  return (
    <div className="App">
      <Question />
      <Choice />
      <Choice />
    </div>
  );
}

export default App;
