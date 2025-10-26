import axios from 'axios';

const apiClient = axios.create({
  baseURL: import.meta.env.DEV ? 'http://localhost:8085/api' : '/api',
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

export const uploadImage = async (file, projectId = null) => {
  const formData = new FormData();
  formData.append('image', file);
  if (projectId) {
    formData.append('project_id', projectId);
  }

  const response = await apiClient.post('/images', formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });
  return response.data;
};

export const getImageUrl = (imageId) => {
  const baseURL = import.meta.env.DEV ? 'http://localhost:8085' : '';
  return `${baseURL}/api/images/${imageId}`;
};

export const listImages = async () => {
  const response = await apiClient.get('/images');
  return response.data;
};

export const deleteImage = async (imageId) => {
  await apiClient.delete(`/images/${imageId}`);
};