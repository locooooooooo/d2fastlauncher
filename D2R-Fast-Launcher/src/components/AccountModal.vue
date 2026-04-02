<script setup lang="ts">
import { ref, computed } from 'vue';
import type { Account } from '../store/account';
import { X } from 'lucide-vue-next';

const props = defineProps<{
  show: boolean;
  editAccount?: Account | null;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save', account: Omit<Account, 'id' | 'order'>): void;
}>();

const isEditing = computed(() => !!props.editAccount);

const formData = ref({
  username: '',
  password: '',
  region: 'kr' as 'kr' | 'eu' | 'us' | 'cn',
  loginMethod: 'auto' as 'auto' | 'bnet',
  mod: '',
  fullscreen: false
});

// 当弹窗打开时，如果是编辑模式，填充数据
const initData = () => {
  if (props.editAccount) {
    formData.value = {
      username: props.editAccount.username,
      password: '', // 密码不回显
      region: props.editAccount.region,
      loginMethod: props.editAccount.loginMethod || 'auto',
      mod: props.editAccount.mod,
      fullscreen: props.editAccount.fullscreen
    };
  } else {
    formData.value = {
      username: '',
      password: '',
      region: 'kr',
      loginMethod: 'auto',
      mod: '',
      fullscreen: false
    };
  }
};

const handleSave = () => {
  if (!formData.value.username) {
    alert('账号名称不能为空');
    return;
  }
  emit('save', { ...formData.value });
  formData.value.password = ''; // 清空密码
};
</script>

<template>
  <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
    <!-- Modal Content -->
    <div class="bg-gray-800 border border-gray-700 rounded-xl shadow-2xl w-full max-w-md overflow-hidden" @vue:mounted="initData">
      
      <!-- Header -->
      <div class="flex justify-between items-center p-6 border-b border-gray-700 bg-gray-800/50">
        <h2 class="text-xl font-bold text-gray-100">
          {{ isEditing ? '编辑账号' : '添加新账号' }}
        </h2>
        <button @click="emit('close')" class="text-gray-400 hover:text-gray-200 transition-colors">
          <X class="w-6 h-6" />
        </button>
      </div>

      <!-- Body -->
      <div class="p-6 space-y-5">
        <!-- Username -->
        <div>
          <label class="block text-sm font-medium text-gray-400 mb-1">账号名称 / 邮箱</label>
          <input 
            v-model="formData.username" 
            type="text" 
            class="w-full bg-gray-900 border border-gray-700 rounded-lg px-4 py-2 text-gray-100 focus:outline-none focus:border-yellow-600 focus:ring-1 focus:ring-yellow-600 transition-all"
            placeholder="example@email.com"
          >
        </div>

        <!-- Password -->
        <div>
          <label class="block text-sm font-medium text-gray-400 mb-1">
            密码 <span v-if="isEditing" class="text-xs text-gray-500">(留空表示不修改)</span>
            <span v-if="formData.loginMethod === 'bnet'" class="text-xs text-yellow-600 ml-2">(战网启动无需填写密码)</span>
          </label>
          <input 
            v-model="formData.password" 
            type="password" 
            :disabled="formData.loginMethod === 'bnet'"
            class="w-full bg-gray-900 border border-gray-700 rounded-lg px-4 py-2 text-gray-100 focus:outline-none focus:border-yellow-600 focus:ring-1 focus:ring-yellow-600 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
            placeholder="输入密码"
          >
        </div>

        <!-- Login Method -->
        <div>
          <label class="block text-sm font-medium text-gray-400 mb-1">启动方式</label>
          <select 
            v-model="formData.loginMethod"
            class="w-full bg-gray-900 border border-gray-700 rounded-lg px-4 py-2 text-gray-100 focus:outline-none focus:border-yellow-600 focus:ring-1 focus:ring-yellow-600 transition-all"
          >
            <option value="auto">直连启动 (需输入密码，不支持验证码)</option>
            <option value="bnet">战网启动 (唤起战网客户端，支持国服/验证码)</option>
          </select>
        </div>

        <!-- Region -->
        <div>
          <label class="block text-sm font-medium text-gray-400 mb-1">游戏区服</label>
          <select 
            v-model="formData.region"
            class="w-full bg-gray-900 border border-gray-700 rounded-lg px-4 py-2 text-gray-100 focus:outline-none focus:border-yellow-600 focus:ring-1 focus:ring-yellow-600 transition-all"
          >
            <option value="cn">国服 (CN)</option>
            <option value="kr">亚洲 (KR)</option>
            <option value="eu">欧洲 (EU)</option>
            <option value="us">美洲 (US)</option>
          </select>
        </div>

        <!-- Mod -->
        <div>
          <label class="block text-sm font-medium text-gray-400 mb-1">Mod 名称 (可选)</label>
          <input 
            v-model="formData.mod" 
            type="text" 
            class="w-full bg-gray-900 border border-gray-700 rounded-lg px-4 py-2 text-gray-100 focus:outline-none focus:border-yellow-600 focus:ring-1 focus:ring-yellow-600 transition-all"
            placeholder="输入 mod 文件夹名称"
          >
        </div>

        <!-- Checkboxes -->
        <div class="flex items-center space-x-3 pt-2">
          <label class="flex items-center space-x-2 cursor-pointer">
            <input 
              v-model="formData.fullscreen" 
              type="checkbox" 
              class="w-4 h-4 rounded border-gray-700 bg-gray-900 text-yellow-600 focus:ring-yellow-600 focus:ring-offset-gray-800"
            >
            <span class="text-sm text-gray-300">全屏运行</span>
          </label>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex justify-end space-x-3 p-6 border-t border-gray-700 bg-gray-800/50">
        <button 
          @click="emit('close')"
          class="px-4 py-2 rounded-lg text-gray-300 hover:bg-gray-700 transition-colors font-medium"
        >
          取消
        </button>
        <button 
          @click="handleSave"
          class="px-6 py-2 rounded-lg bg-yellow-600 hover:bg-yellow-500 text-gray-900 font-bold transition-colors shadow-lg"
        >
          保存
        </button>
      </div>

    </div>
  </div>
</template>
