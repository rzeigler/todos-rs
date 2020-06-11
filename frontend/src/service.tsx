import axios from 'axios';
import { Todo } from './state';

export function getTodos(): Promise<Todo[]> {
  return axios.get("/todos")
    .then(r => r.data);
}

export function createTodo(name: string = "", completed: boolean = false): Promise<Todo> {
  return axios.post("/todos", { name, completed })
    .then(r => r.data)
}

export function putTodo(todo: Todo): Promise<Todo> {
  return axios.put(`/todos/${todo.id}`, todo)
    .then(r => r.data);
}
