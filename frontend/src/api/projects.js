
import axios from 'axios';

const apiClient = axios.create({
  baseURL: import.meta.env.DEV ? 'http://localhost:8085/api' : '/api',
  headers: {
    'Content-Type': 'application/json',
  },
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
