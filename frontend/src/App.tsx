import React from 'react';
import './App.css';
import Choice from './components/Choice/choice'
import Question from './components/Question/question'
import Login from './components/Login/login'

function App() {
  return (
    <div className="App">
      <Login />
      <Question />
      <Choice />
      <Choice />
    </div>
  );
}

export default App;
