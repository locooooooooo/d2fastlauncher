<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useAccountStore, type Account } from '../store/account';
import AccountCard from '../components/AccountCard.vue';
import AccountModal from '../components/AccountModal.vue';
import { Plus, Rocket, Settings as SettingsIcon, Unlock } from 'lucide-vue-next';
import { invoke } from '@tauri-apps/api/tauri';

const accountStore = useAccountStore();

const showModal = ref(false);
const editingAccountId = ref<string | null>(null);
const d2rPath = ref<string>('');

onMounted(async () => {
  try {
    // 尝试获取 D2R 安装路径
    d2rPath.value = await invoke('get_d2r_path');
    console.log('Found D2R Path:', d2rPath.value);
  } catch (error) {
    console.error('Failed to get D2R path:', error);
  }
});

const currentEditAccount = computed(() => {
  if (!editingAccountId.value) return null;
  return accountStore.accounts.find(a => a.id === editingAccountId.value) || null;
});

const handleAddClick = () => {
  editingAccountId.value = null;
  showModal.value = true;
};

const handleEditClick = (id: string) => {
  editingAccountId.value = id;
  showModal.value = true;
};

const handleDeleteClick = (id: string) => {
  if (confirm('确定要删除这个账号配置吗？')) {
    accountStore.deleteAccount(id);
  }
};

const handleLaunch = async (id: string) => {
  const account = accountStore.accounts.find(a => a.id === id);
  if (account) {
    if (!d2rPath.value) {
      alert("未找到 D2R 安装路径，请检查注册表或手动配置");
      return;
    }
    
    try {
      // 1. 先尝试杀 Mutex (多开核心)
      await invoke('kill_d2r_mutex');
      
      // 兼容老数据，如果没有 loginMethod 字段或者显式指定为 bnet，则走战网模式
      // 对于国服，通常强制要求使用战网模式
      const isBnet = account.loginMethod === 'bnet' || (account.region === 'cn' && !account.loginMethod);

      if (isBnet) {
        // 战网启动模式
        const result = await invoke('launch_bnet');
        console.log('Bnet Launch result:', result);
        alert('已唤起战网，请手动在新的战网窗口中登录并点击【进入游戏】。\n游戏启动后，若需继续多开，请点击顶部的【一键解除多开限制】按钮，然后再次启动下一个账号。');
      } else {
        // 直连启动模式 (带参数)
        const args: string[] = [];
        if (!account.fullscreen) {
          args.push("-w");
        }
        if (account.mod) {
          args.push("-mod", account.mod);
        }
        // 根据区服设置 address
        if (account.region === 'kr') args.push("-address", "kr.actual.battle.net");
        else if (account.region === 'eu') args.push("-address", "eu.actual.battle.net");
        else if (account.region === 'us') args.push("-address", "us.actual.battle.net");
        else if (account.region === 'cn') args.push("-address", "cn.actual.battle.net"); // 国服可能无效，但保留逻辑

        const result = await invoke('launch_d2r', { 
          path: d2rPath.value,
          args: args,
          username: account.username,
          password: account.password || null
        });
        console.log('Launch result:', result);
      }
    } catch (error) {
      console.error('Launch failed:', error);
      alert(`启动失败: ${error}`);
    }
  }
};

const handleKillMutex = async () => {
  try {
    const result = await invoke('kill_d2r_mutex');
    alert(`解除多开限制成功！\n${result}`);
  } catch (error) {
    alert(`解除多开限制失败: ${error}`);
  }
};

const handleLaunchAll = async () => {
  if (accountStore.accounts.length === 0) return;
  console.log('Launching all accounts...');
  
  for (const account of accountStore.accounts) {
    await handleLaunch(account.id);
    // 可选：添加一点延迟，避免同时启动导致卡顿
    await new Promise(resolve => setTimeout(resolve, 2000));
  }
};

const handleSaveAccount = (accountData: Omit<Account, 'id' | 'order'>) => {
  if (editingAccountId.value) {
    accountStore.updateAccount(editingAccountId.value, accountData);
  } else {
    accountStore.addAccount(accountData);
  }
  showModal.value = false;
};
</script>

<template>
  <div class="min-h-screen bg-gray-900 text-gray-100 flex flex-col">
    <!-- Header -->
    <header class="bg-gray-800 border-b border-gray-700 sticky top-0 z-10 shadow-md">
      <div class="max-w-6xl mx-auto px-6 h-16 flex items-center justify-between">
        <div class="flex items-center space-x-3">
          <!-- Logo Placeholder -->
          <div class="w-8 h-8 bg-yellow-600 rounded-md flex items-center justify-center font-bold text-gray-900">
            D2
          </div>
          <h1 class="text-xl font-bold tracking-wider text-gray-100">
            Fast Launcher <span class="text-yellow-600 text-sm ml-1">v2.0</span>
          </h1>
        </div>
        
        <div class="flex items-center space-x-4">
          <button 
            @click="handleKillMutex"
            class="flex items-center space-x-2 bg-gray-700 hover:bg-gray-600 text-gray-200 border border-gray-600 font-bold py-2 px-4 rounded-lg transition-colors shadow-sm"
          >
            <Unlock class="w-4 h-4" />
            <span>解除多开限制</span>
          </button>
          <button 
            @click="handleLaunchAll"
            class="flex items-center space-x-2 bg-yellow-600 hover:bg-yellow-500 text-gray-900 font-bold py-2 px-4 rounded-lg transition-colors shadow-lg"
          >
            <Rocket class="w-4 h-4" />
            <span>一键全开</span>
          </button>
          <button class="p-2 text-gray-400 hover:text-gray-100 transition-colors" title="全局设置">
            <SettingsIcon class="w-5 h-5" />
          </button>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <main class="flex-1 max-w-6xl mx-auto w-full p-6">
      
      <!-- Account Grid -->
      <div v-if="accountStore.accounts.length > 0" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <AccountCard 
          v-for="account in accountStore.accounts" 
          :key="account.id"
          :account="account"
          @launch="handleLaunch"
          @edit="handleEditClick"
          @delete="handleDeleteClick"
        />
        
        <!-- Add New Card Button -->
        <button 
          @click="handleAddClick"
          class="border-2 border-dashed border-gray-700 rounded-xl p-4 flex flex-col items-center justify-center text-gray-500 hover:text-yellow-500 hover:border-yellow-600 hover:bg-gray-800/50 transition-all min-h-[160px] group"
        >
          <div class="w-12 h-12 rounded-full bg-gray-800 group-hover:bg-yellow-600/20 flex items-center justify-center mb-3 transition-colors">
            <Plus class="w-6 h-6" />
          </div>
          <span class="font-medium">添加新账号</span>
        </button>
      </div>

      <!-- Empty State -->
      <div v-else class="flex flex-col items-center justify-center py-32 text-gray-500">
        <div class="w-20 h-20 bg-gray-800 rounded-full flex items-center justify-center mb-6">
          <Plus class="w-10 h-10 text-gray-600" />
        </div>
        <h2 class="text-xl font-bold text-gray-300 mb-2">还没有添加任何账号</h2>
        <p class="mb-8">点击下方按钮添加你的第一个 D2R 战网账号配置。</p>
        <button 
          @click="handleAddClick"
          class="flex items-center space-x-2 bg-gray-800 hover:bg-gray-700 text-gray-200 border border-gray-600 font-semibold py-3 px-6 rounded-lg transition-colors"
        >
          <Plus class="w-5 h-5" />
          <span>添加账号配置</span>
        </button>
      </div>

    </main>

    <!-- Modals -->
    <AccountModal 
      :show="showModal" 
      :edit-account="currentEditAccount"
      @close="showModal = false"
      @save="handleSaveAccount"
    />
  </div>
</template>
