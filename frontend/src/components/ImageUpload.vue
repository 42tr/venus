<template>
  <div class="image-upload">
    <input
      ref="fileInput"
      type="file"
      accept="image/*"
      @change="handleFileSelect"
      style="display: none"
      multiple
    />
    
    <div class="upload-area" @click="triggerFileSelect" @drop="handleDrop" @dragover.prevent>
      <div class="upload-content">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
          <circle cx="8.5" cy="8.5" r="1.5"/>
          <polyline points="21,15 16,10 5,21"/>
        </svg>
        <p>点击或拖拽上传图片</p>
        <p class="hint">支持 JPG、PNG、GIF 格式</p>
      </div>
    </div>

    <div v-if="uploading" class="uploading">
      <div class="progress-bar">
        <div class="progress" :style="{ width: uploadProgress + '%' }"></div>
      </div>
      <p>上传中... {{ uploadProgress }}%</p>
    </div>

    <div v-if="uploadedImages.length > 0" class="uploaded-images">
      <h3>已上传的图片</h3>
      <div class="image-grid">
        <div
          v-for="image in uploadedImages"
          :key="image.id"
          class="image-item"
          @click="$emit('image-selected', image)"
        >
          <LazyImage
            :src="image.url"
            :alt="image.original_name"
            class="thumbnail"
          />
          <div class="image-info">
            <p class="image-name">{{ image.original_name }}</p>
            <p class="image-size">{{ formatFileSize(image.size) }}</p>
          </div>
          <button @click.stop="deleteImage(image.id)" class="delete-btn">×</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { uploadImage, listImages, deleteImage as deleteImageApi } from '../api/images';
import LazyImage from './LazyImage.vue';

const emit = defineEmits(['image-selected']);

const fileInput = ref(null);
const uploading = ref(false);
const uploadProgress = ref(0);
const uploadedImages = ref([]);

onMounted(async () => {
  await loadImages();
});

const loadImages = async () => {
  try {
    uploadedImages.value = await listImages();
  } catch (error) {
    console.error('加载图片失败:', error);
  }
};

const triggerFileSelect = () => {
  fileInput.value?.click();
};

const handleFileSelect = (event) => {
  const files = Array.from(event.target.files);
  uploadFiles(files);
};

const handleDrop = (event) => {
  event.preventDefault();
  const files = Array.from(event.dataTransfer.files).filter(file => 
    file.type.startsWith('image/')
  );
  uploadFiles(files);
};

const uploadFiles = async (files) => {
  if (files.length === 0) return;

  uploading.value = true;
  uploadProgress.value = 0;

  try {
    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      const uploadedImage = await uploadImage(file);
      uploadedImages.value.unshift(uploadedImage);
      
      uploadProgress.value = Math.round(((i + 1) / files.length) * 100);
    }
  } catch (error) {
    console.error('上传失败:', error);
    alert('上传失败，请重试');
  } finally {
    uploading.value = false;
    uploadProgress.value = 0;
    // 清空文件输入
    if (fileInput.value) {
      fileInput.value.value = '';
    }
  }
};

const deleteImage = async (imageId) => {
  if (!confirm('确定要删除这张图片吗？')) return;

  try {
    await deleteImageApi(imageId);
    // 立即从列表中移除，提供即时反馈
    uploadedImages.value = uploadedImages.value.filter(img => img.id !== imageId);
    console.log('图片删除成功');
  } catch (error) {
    console.error('删除失败:', error);
    alert('删除失败，请重试');
    // 如果删除失败，重新加载列表以确保状态一致
    await loadImages();
  }
};

const formatFileSize = (bytes) => {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};
</script>

<style scoped>
.image-upload {
  padding: 1rem;
}

.upload-area {
  border: 2px dashed #ddd;
  border-radius: 8px;
  padding: 2rem;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.3s;
}

.upload-area:hover {
  border-color: #007bff;
}

.upload-content svg {
  color: #666;
  margin-bottom: 1rem;
}

.upload-content p {
  margin: 0.5rem 0;
  color: #666;
}

.hint {
  font-size: 0.875rem;
  color: #999;
}

.uploading {
  margin: 1rem 0;
  text-align: center;
}

.progress-bar {
  width: 100%;
  height: 8px;
  background: #f0f0f0;
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 0.5rem;
}

.progress {
  height: 100%;
  background: #007bff;
  transition: width 0.3s;
}

.uploaded-images {
  margin-top: 2rem;
}

.uploaded-images h3 {
  margin-bottom: 1rem;
  color: #333;
}

.image-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 1rem;
}

.image-item {
  position: relative;
  border: 1px solid #ddd;
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
}

.image-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.thumbnail {
  width: 100%;
  height: 150px;
  object-fit: cover;
}

.image-info {
  padding: 0.75rem;
}

.image-name {
  margin: 0 0 0.25rem 0;
  font-weight: 500;
  font-size: 0.875rem;
  color: #333;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.image-size {
  margin: 0;
  font-size: 0.75rem;
  color: #666;
}

.delete-btn {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
  width: 24px;
  height: 24px;
  background: rgba(220, 53, 69, 0.9);
  color: white;
  border: none;
  border-radius: 50%;
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.delete-btn:hover {
  background: rgba(220, 53, 69, 1);
}
</style>