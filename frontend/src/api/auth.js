import axios from 'axios';

const API_BASE = '/api/auth';

// 创建axios实例
const authApi = axios.create({
  baseURL: API_BASE,
  headers: {
    'Content-Type': 'application/json',
  },
});

// 请求拦截器 - 添加token
authApi.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

export const register = async (userData) => {
  const response = await authApi.post('/register', userData);
  return response.data;
};

export const login = async (credentials) => {
  const response = await authApi.post('/login', credentials);
  return response.data;
};

export const getCurrentUser = async () => {
  const response = await authApi.get('/user');
  return response.data;
};

export const logout = () => {
  localStorage.removeItem('token');
  localStorage.removeItem('user');
};