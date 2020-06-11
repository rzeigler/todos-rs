import React from 'react';
import { appStore, fetchTodos } from './state';
import { Provider } from 'react-redux';
import Body from './Body';
import './App.css';

appStore.dispatch(fetchTodos());

function App() {
  return (
    <Provider store={appStore}>
      <Body />
    </Provider>
  );
}

export default App;
