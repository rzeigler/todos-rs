
import { createReducer, createAction, configureStore, ThunkDispatch, Action } from '@reduxjs/toolkit';
import { getTodos, putTodo } from './service';

export type Todo = {
  id: number;
  name: string;
  completed: boolean;
};

export type AppState = {
  todos: Todo[] | null;
  active: number | null;
};

export const requestTodos = createAction('requestTodos');
export const receiveTodos = createAction<Todo[]>('receiveTodos');
export const activateTodo = createAction<number>('activateTodo');
export const deactivateTodo = createAction<number>('deactivateTodo');

const initial: Todo[] = [{
  id: 1,
  name: "#1",
  completed: false
}, {
  id: 2,
  name: "#2",
  completed: false
}]

const appReducer = createReducer({ todos: initial, active: null } as AppState, builder => {
  builder.addCase(requestTodos, (_state, _action) => ({ todos: null, active: null }))
    .addCase(receiveTodos, (state, action) => ({ ...state, todos: action.payload }))
    .addCase(activateTodo, (state, action) => ({ ...state, active: action.payload }))
    .addCase(deactivateTodo, (state, action) =>
      state.active === action.payload ?
        { ...state, active: null } : state)
})

export function fetchTodos() {
  return (dispatch: ThunkDispatch<AppState, void, Action>) => {
    dispatch(requestTodos)
    getTodos()
      .then((todos) => dispatch(receiveTodos(todos)))
  }
}

export function setName(id: number, name: string) {
  return (dispatch: ThunkDispatch<AppState, void, Action>, getState: () => AppState) => {
    const todo = getState().todos?.find(t => t.id === id);
    if (todo) {
      putTodo({ ...todo, name })
        .then((result) => {
          const todos = (getState().todos || []);
          const idx = todos.findIndex(t => t.id === id);
          const replaced = todos.slice(0, idx).concat([result]).concat(todos.slice(idx + 1));
          dispatch(receiveTodos(replaced));
        })
    }
  }
}

export function setCompleted(id: number, completed: boolean) {
  return (dispatch: ThunkDispatch<AppState, void, Action>, getState: () => AppState) => {
    const todo = getState().todos?.find(t => t.id === id);
    if (todo) {
      putTodo({ ...todo, completed })
        .then((result) => {
          const todos = (getState().todos || []);
          const idx = todos.findIndex(t => t.id === id);
          const replaced = todos.slice(0, idx).concat([result]).concat(todos.slice(idx + 1));
          dispatch(receiveTodos(replaced));
        })
    }
  }
}

export const appStore = configureStore({
  reducer: appReducer
});

