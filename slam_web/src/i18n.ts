export type Lang = 'zh' | 'en';

export const LANGUAGE_NAMES: Record<Lang, string> = {
  zh: '中文',
  en: 'English',
};

export const TEXTS = {
  zh: {
    home: {
      headTitle: '首页',
      motion: '运动',
      stats: '统计',
      settings: '设置',
      greetings: {
        morning: '早上好',
        noon: '中午好',
        afternoon: '下午好',
        evening: '晚上好',
      },
      language: '语言',
      logout: '退出登录',
      labels: {
        time: '时间',
        distance: '距离',
        calories: '卡路里',
      },
      miImport: '数据导入',
      xiaomiSports: '小米运动',
    },
    addsports: {
      headTitle: '添加运动',
      title: '记录运动',
      imagesTitle: '运动图片',
      aiButton: 'AI识别',
      pickImages: '选择运动图片',
      manualButton: '手动录入',
      submitBasicTitle: '基本信息',
      submitTypeLabel: '运动类型',
      submitStartTimeLabel: '开始时间',
      submitCaloriesLabel: '卡路里',
      submitDistanceLabel: '距离(米)',
      submitDurationLabel: '持续时间',
      submitPaceLabel: '配速',
      submitHRAvgLabel: '心率(均值)',
      submitHRMaxLabel: '心率(最大)',
      submitSwimTitle: '专项(游泳)',
      submitStrokeLabel: '泳姿',
      submitStrokeAvgLabel: '划水次数',
      submitSwolfAvgLabel: 'SWOLF',
      submitTracksTitle: '分段',
      submitTrackDelete: '删除',
      submitTrackAdd: '添加分段',
      submitButton: '提交',
      optUnknown: '未知',
      optSwimming: '游泳',
      optRunning: '跑步',
      optCycling: '骑行',
      strokeUnknown: '未知',
      strokeFreestyle: '自由泳',
      strokeButterfly: '蝶泳',
      strokeBreaststroke: '蛙泳',
      strokeBackstroke: '仰泳',
      strokeMedley: '混合泳',
    },
    login: {
      headTitle: '登录',
      title: '登录',
      username: '用户名',
      password: '密码',
      login: '登录',
      register: '注册',
      language: '语言',
    },
    register: {
      headTitle: '注册',
      title: '注册',
      username: '用户名',
      nickname: '昵称',
      password: '密码',
      confirm: '确认密码',
      submit: '提交注册',
      cancel: '取消',
      language: '语言',
      errorFill: '请填写完整信息',
      errorLength: '密码长度至少6位',
      errorMismatch: '两次输入的密码不一致',
      success: '注册成功',
    },
  },
  en: {
    home: {
      headTitle: 'Home',
      motion: 'Sports',
      stats: 'Stats',
      settings: 'Settings',
      greetings: {
        morning: 'Good morning',
        noon: 'Good noon',
        afternoon: 'Good afternoon',
        evening: 'Good evening',
      },
      language: 'Language',
      logout: 'Logout',
      labels: {
        time: 'Time',
        distance: 'Distance',
        calories: 'Calories',
      },
      miImport: 'Data Import',
      xiaomiSports: 'Xiaomi Sports',
    },
    addsports: {
      headTitle: 'Add Sports',
      title: 'Record',
      imagesTitle: 'Sport Images',
      aiButton: 'AI Recognition',
      pickImages: 'Pick Sport Images',
      manualButton: 'Manual',
      submitBasicTitle: 'Basic Info',
      submitTypeLabel: 'Sport Type',
      submitStartTimeLabel: 'Start Time',
      submitCaloriesLabel: 'Calories',
      submitDistanceLabel: 'Distance (m)',
      submitDurationLabel: 'Duration',
      submitPaceLabel: 'Pace',
      submitHRAvgLabel: 'Avg HR',
      submitHRMaxLabel: 'Max HR',
      submitSwimTitle: 'Swimming',
      submitStrokeLabel: 'Stroke',
      submitStrokeAvgLabel: 'StrokeCount',
      submitSwolfAvgLabel: 'SWOLF',
      submitTracksTitle: 'Segments',
      submitTrackDelete: 'Delete',
      submitTrackAdd: 'Add Segment',
      submitButton: 'Submit',
      optUnknown: 'Unknown',
      optSwimming: 'Swimming',
      optRunning: 'Running',
      optCycling: 'Cycling',
      strokeUnknown: 'Unknown',
      strokeFreestyle: 'Freestyle',
      strokeButterfly: 'Butterfly',
      strokeBreaststroke: 'Breaststroke',
      strokeBackstroke: 'Backstroke',
      strokeMedley: 'Medley',
    },
    login: {
      headTitle: 'Login',
      title: 'Login',
      username: 'Username',
      password: 'Password',
      login: 'Login',
      register: 'Register',
      language: 'Language',
    },
    register: {
      headTitle: 'Register',
      title: 'Register',
      username: 'Username',
      nickname: 'Nickname',
      password: 'Password',
      confirm: 'Confirm Password',
      submit: 'Submit',
      cancel: 'Cancel',
      language: 'Language',
      errorFill: 'Please fill in all fields',
      errorLength: 'Password must be at least 6 characters',
      errorMismatch: 'Passwords do not match',
      success: 'Registration successful',
    },
  },
} as const;

export function getSavedLang(): Lang {
  try {
    const saved = localStorage.getItem('lang');
    if (saved === 'zh' || saved === 'en') {
      return saved;
    }
  } catch {}
  return 'zh';
}

export function saveLang(lang: Lang): void {
  try {
    localStorage.setItem('lang', lang);
  } catch {}
}
