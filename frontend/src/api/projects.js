
import axios from 'axios';
import getApiConfig from '../config/api.js';

const apiConfig = getApiConfig();

const apiClient = axios.create({
  baseURL: apiConfig.baseURL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// 请求拦截器 - 添加token
apiClient.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

export const getProjects = async () => {
  const response = await apiClient.get('/projects');
  return response.data;
};

export const getProjectById = async (id) => {
  const response = await apiClient.get(`/projects/${id}`);
  return response.data;
};

export const createProject = async (project) => {
  const response = await apiClient.post('/projects', project);
  return response.data;
};

export const updateProject = async (id, data) => {
  const response = await apiClient.put(`/projects/${id}`, data);
  return response.data;
};

export const deleteProject = async (id) => {
  await apiClient.delete(`/projects/${id}`);
};
