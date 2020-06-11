import React from 'react';
import { AppState, activateTodo, deactivateTodo, setName, setCompleted } from './state';
import { connect } from 'react-redux';

import './Todo.css'
import { ThunkDispatch, Action } from '@reduxjs/toolkit';

type TodoStateToProps = {
  // Which todo is active
  activated: number | null;

  id: number;
  name: string;
  completed: boolean;
}

type TodoDispatchToProps = {
  setFocus(id: number): void;
  clearFocus(ev: React.KeyboardEvent<HTMLInputElement>, id: number): void;
  setName(id: number, name: string): void;
  setCompleted(id: number, completed: boolean): void;
}

type TodoOwnProps = {
  index: number;
}

type TodoProps = TodoOwnProps & TodoStateToProps & TodoDispatchToProps

function Todo(props: TodoProps) {
  return (
    <div className="Todo-item">
      <input type="checkbox" checked={props.completed} onChange={(ev) => props.setCompleted(props.id, ev.target.checked)} />
      {props.activated === props.id ?
        <input
          type="text"
          value={props.name}
          onKeyPress={(ev) => props.clearFocus(ev, props.id)}
          onChange={(ev) => props.setName(props.id, ev.target.value)} /> :
        <span onClick={() => props.setFocus(props.id)}>{props.name}</span>
      }
    </div>
  )
}

function mapStateToProps(state: AppState, ownProps: TodoOwnProps): TodoStateToProps {
  return { ...(state.todos || [])[ownProps.index], activated: state.active };
}

function mapDispatchToProps(dispatch: ThunkDispatch<AppState, void, Action>): TodoDispatchToProps {
  return {
    setFocus: (id) => dispatch(activateTodo(id)),
    clearFocus: (ev, id) => {
      if (ev.key === 'Enter' || ev.key === 'Escape')
        dispatch(deactivateTodo(id))
    },
    setName: (id, name) => dispatch(setName(id, name)),
    setCompleted: (id, completed) => dispatch(setCompleted(id, completed))
  }
}

export default connect(mapStateToProps, mapDispatchToProps)(Todo)
