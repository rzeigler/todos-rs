import React from 'react';
import { Todo as TodoItem, AppState } from './state';
import Todo from './Todo';
import { connect } from 'react-redux';

type BodyStateToProps = {
  todos: readonly TodoItem[];
}

type BodyProps = BodyStateToProps

function Body(props: BodyProps) {
  return (
    <div className="App">
      <header className="App-header">Todos</header>
      <div style={{ "width": "400px", "margin": "auto" }}>
        {props.todos.map((todo, index) =>
          <Todo key={todo.id} index={index} />
        )}
      </div>
    </div >
  );
}

function mapStateToProps(state: AppState): BodyStateToProps {
  return {
    todos: state.todos || []
  };
}

export default connect(mapStateToProps)(Body);
