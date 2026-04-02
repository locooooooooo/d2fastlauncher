import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

export interface Account {
  id: string;
  username: string;
  password?: string; // 仅在编辑或启动时解密使用
  region: 'kr' | 'eu' | 'us' | 'cn';
  loginMethod: 'auto' | 'bnet'; // auto: 直连启动(不支持验证码), bnet: 战网启动(支持国服/验证码)
  mod: string;
  fullscreen: boolean;
  order: number;
}

export const useAccountStore = defineStore('account', () => {
  // 尝试从 localStorage 加载数据，如果没有则使用默认模拟数据
  const loadInitialData = (): Account[] => {
    const saved = localStorage.getItem('d2r_accounts');
    if (saved) {
      try {
        return JSON.parse(saved);
      } catch (e) {
        console.error('Failed to parse saved accounts:', e);
      }
    }
    return [
        {
          id: '1',
          username: 'PlayerOne',
          region: 'kr',
          loginMethod: 'auto',
          mod: '',
          fullscreen: false,
          order: 1
        },
        {
          id: '2',
          username: 'SmurfAcc',
          region: 'kr',
          loginMethod: 'bnet',
          mod: 'basemod',
          fullscreen: false,
          order: 2
        }
      ];
  };

  const accounts = ref<Account[]>(loadInitialData());

  // 监听数据变化并保存到 localStorage
  watch(accounts, (newAccounts) => {
    localStorage.setItem('d2r_accounts', JSON.stringify(newAccounts));
  }, { deep: true });

  const addAccount = (account: Omit<Account, 'id' | 'order'>) => {
    const newId = Date.now().toString();
    accounts.value.push({
      ...account,
      id: newId,
      order: accounts.value.length + 1
    });
  };

  const updateAccount = (id: string, updatedData: Partial<Account>) => {
    const index = accounts.value.findIndex(a => a.id === id);
    if (index !== -1) {
      accounts.value[index] = { ...accounts.value[index], ...updatedData };
    }
  };

  const deleteAccount = (id: string) => {
    accounts.value = accounts.value.filter(a => a.id !== id);
    // 重新排序
    accounts.value.forEach((a, index) => {
      a.order = index + 1;
    });
  };

  return {
    accounts,
    addAccount,
    updateAccount,
    deleteAccount
  };
});
