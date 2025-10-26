<template>
  <div class="auth-container">
    <div class="auth-form">
      <h1>Venus</h1>
      <div class="form-tabs">
        <button 
          :class="{ active: isLogin }" 
          @click="isLogin = true"
        >
          登录
        </button>
        <button 
          :class="{ active: !isLogin }" 
          @click="isLogin = false"
        >
          注册
        </button>
      </div>

      <form @submit.prevent="handleSubmit">
        <div class="form-group">
          <label for="username">用户名</label>
          <input 
            id="username"
            v-model="form.username" 
            type="text" 
            required 
            placeholder="请输入用户名"
          />
        </div>

        <div v-if="!isLogin" class="form-group">
          <label for="email">邮箱</label>
          <input 
            id="email"
            v-model="form.email" 
            type="email" 
            required 
            placeholder="请输入邮箱"
          />
        </div>

        <div class="form-group">
          <label for="password">密码</label>
          <input 
            id="password"
            v-model="form.password" 
            type="password" 
            required 
            placeholder="请输入密码"
          />
        </div>

        <button 
          type="submit" 
          class="submit-btn"
          :disabled="isLoading"
        >
          {{ isLoading ? '处理中...' : (isLogin ? '登录' : '注册') }}
        </button>
      </form>

      <div v-if="error" class="error-message">
        {{ error }}
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive } from 'vue';
import { login, register } from '../api/auth';

const emit = defineEmits(['auth-success']);

const isLogin = ref(true);
const isLoading = ref(false);
const error = ref('');

const form = reactive({
  username: '',
  email: '',
  password: ''
});

const handleSubmit = async () => {
  if (isLoading.value) return;
  
  isLoading.value = true;
  error.value = '';

  try {
    let response;
    if (isLogin.value) {
      response = await login({
        username: form.username,
        password: form.password
      });
    } else {
      response = await register({
        username: form.username,
        email: form.email,
        password: form.password
      });
    }

    // 保存token
    localStorage.setItem('token', response.token);
    localStorage.setItem('user', JSON.stringify(response.user));
    
    emit('auth-success', response);
  } catch (err) {
    console.error('认证失败:', err);
    if (err.response?.status === 409) {
      error.value = '用户名或邮箱已存在';
    } else if (err.response?.status === 401) {
      error.value = '用户名或密码错误';
    } else {
      error.value = '操作失败，请稍后重试';
    }
  } finally {
    isLoading.value = false;
  }
};
</script>

<style scoped>
.auth-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.auth-form {
  background: white;
  padding: 2rem;
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  width: 100%;
  max-width: 400px;
}

h1 {
  text-align: center;
  margin-bottom: 1.5rem;
  color: #333;
  font-size: 2rem;
}

.form-tabs {
  display: flex;
  margin-bottom: 1.5rem;
  border-bottom: 1px solid #e0e0e0;
}

.form-tabs button {
  flex: 1;
  padding: 0.75rem;
  border: none;
  background: none;
  cursor: pointer;
  color: #666;
  font-size: 1rem;
  transition: all 0.3s ease;
}

.form-tabs button.active {
  color: #667eea;
  border-bottom: 2px solid #667eea;
}

.form-group {
  margin-bottom: 1rem;
}

label {
  display: block;
  margin-bottom: 0.5rem;
  color: #333;
  font-weight: 500;
}

input {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 1rem;
  transition: border-color 0.3s ease;
  box-sizing: border-box;
}

input:focus {
  outline: none;
  border-color: #667eea;
}

.submit-btn {
  width: 100%;
  padding: 0.75rem;
  background: #667eea;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 1rem;
  cursor: pointer;
  transition: background-color 0.3s ease;
}

.submit-btn:hover:not(:disabled) {
  background: #5a6fd8;
}

.submit-btn:disabled {
  background: #ccc;
  cursor: not-allowed;
}

.error-message {
  margin-top: 1rem;
  padding: 0.75rem;
  background: #fee;
  color: #c33;
  border-radius: 4px;
  text-align: center;
}
</style>