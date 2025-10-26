// API Configuration
const getApiConfig = () => {
  // Get base URL from environment variable
  const baseURL = import.meta.env.VITE_API_URL || (
    import.meta.env.DEV ? 'http://localhost:8085' : ''
  );

  return {
    baseURL: baseURL ? `${baseURL}/api` : '/api',
    imageBaseURL: baseURL,
  };
};

export default getApiConfig;