<script setup lang="ts">
import { Play, Settings, Trash2 } from 'lucide-vue-next';
import type { Account } from '../store/account';

const { account } = defineProps<{
    account: Account
}>();

const emit = defineEmits<{
  (e: 'launch', id: string): void;
  (e: 'edit', id: string): void;
  (e: 'delete', id: string): void;
}>();

const regionLabels = {
  cn: '国服 (CN)',
  kr: '亚洲 (KR)',
  eu: '欧洲 (EU)',
  us: '美洲 (US)'
};

const loginMethodLabels = {
  auto: '直连启动',
  bnet: '战网启动'
};
</script>

<template>
  <div class="bg-gray-800 border border-gray-700 rounded-xl p-4 hover:border-yellow-600 transition-colors group relative shadow-lg">
    <!-- Header: Username & Status -->
    <div class="flex justify-between items-center mb-4">
      <h3 class="text-xl font-bold text-gray-100 truncate pr-4">{{ account.username }}</h3>
      <div class="flex space-x-2 opacity-0 group-hover:opacity-100 transition-opacity absolute right-4 top-4">
        <button @click="emit('edit', account.id)" class="text-gray-400 hover:text-yellow-500 transition-colors" title="编辑">
          <Settings class="w-5 h-5" />
        </button>
        <button @click="emit('delete', account.id)" class="text-gray-400 hover:text-red-500 transition-colors" title="删除">
          <Trash2 class="w-5 h-5" />
        </button>
      </div>
    </div>

    <!-- Details -->
    <div class="space-y-2 text-sm text-gray-400 mb-6">
      <div class="flex items-center">
        <span class="w-16">启动:</span>
        <span class="text-gray-200">{{ loginMethodLabels[account.loginMethod || 'auto'] }}</span>
      </div>
      <div class="flex items-center">
        <span class="w-16">区服:</span>
        <span class="text-gray-200">{{ regionLabels[account.region] }}</span>
      </div>
      <div class="flex items-center">
        <span class="w-16">Mod:</span>
        <span class="text-gray-200">{{ account.mod || '无' }}</span>
      </div>
      <div class="flex items-center">
        <span class="w-16">窗口:</span>
        <span class="text-gray-200">{{ account.fullscreen ? '全屏' : '窗口化' }}</span>
      </div>
    </div>

    <!-- Launch Button -->
    <button 
      @click="emit('launch', account.id)"
      class="w-full flex items-center justify-center space-x-2 bg-gray-700 hover:bg-yellow-600 hover:text-gray-900 text-gray-300 font-semibold py-2 px-4 rounded-lg transition-all duration-200"
    >
      <Play class="w-4 h-4" />
      <span>启动游戏</span>
    </button>
  </div>
</template>
