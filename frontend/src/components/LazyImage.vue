<template>
  <div class="lazy-image-container">
    <img
      v-if="loaded"
      :src="src"
      :alt="alt"
      :class="className"
      @load="onLoad"
      @error="onError"
    />
    <div v-else-if="loading" class="loading-placeholder">
      <div class="spinner"></div>
    </div>
    <div v-else-if="error" class="error-placeholder">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor">
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
        <circle cx="8.5" cy="8.5" r="1.5"/>
        <polyline points="21,15 16,10 5,21"/>
      </svg>
      <p>加载失败</p>
    </div>
    <div v-else class="placeholder">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor">
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
        <circle cx="8.5" cy="8.5" r="1.5"/>
        <polyline points="21,15 16,10 5,21"/>
      </svg>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';

const props = defineProps({
  src: {
    type: String,
    required: true,
  },
  alt: {
    type: String,
    default: '',
  },
  className: {
    type: String,
    default: '',
  },
});

const loading = ref(false);
const loaded = ref(false);
const error = ref(false);
const observer = ref(null);
const containerRef = ref(null);

onMounted(() => {
  // 使用 Intersection Observer 实现懒加载
  if ('IntersectionObserver' in window) {
    observer.value = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            loadImage();
            observer.value?.unobserve(entry.target);
          }
        });
      },
      {
        rootMargin: '50px', // 提前50px开始加载
      }
    );

    if (containerRef.value) {
      observer.value.observe(containerRef.value);
    }
  } else {
    // 如果不支持 Intersection Observer，直接加载
    loadImage();
  }
});

onUnmounted(() => {
  if (observer.value) {
    observer.value.disconnect();
  }
});

const loadImage = () => {
  if (loading.value || loaded.value) return;
  
  loading.value = true;
  error.value = false;

  const img = new Image();
  img.onload = () => {
    loading.value = false;
    loaded.value = true;
  };
  img.onerror = () => {
    loading.value = false;
    error.value = true;
  };
  img.src = props.src;
};

const onLoad = () => {
  loaded.value = true;
  loading.value = false;
};

const onError = () => {
  error.value = true;
  loading.value = false;
};
</script>

<script>
export default {
  name: 'LazyImage',
};
</script>

<style scoped>
.lazy-image-container {
  position: relative;
  display: inline-block;
  width: 100%;
  height: 100%;
}

.loading-placeholder,
.error-placeholder,
.placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  background: #f8f9fa;
  color: #6c757d;
  min-height: 100px;
}

.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid #f3f3f3;
  border-top: 2px solid #007bff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.error-placeholder {
  background: #f8d7da;
  color: #721c24;
}

.error-placeholder p {
  margin: 0.5rem 0 0 0;
  font-size: 0.875rem;
}

img {
  max-width: 100%;
  height: auto;
  display: block;
}
</style>