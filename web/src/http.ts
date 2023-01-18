import axios from 'axios';
const client = axios.create({
  baseURL: 'http://192.168.1.236:8080/api',
});

const methods = {
  get: client.get,
  post: client.post,
  put: client.put,
  delete: client.delete,
};

export default methods;
