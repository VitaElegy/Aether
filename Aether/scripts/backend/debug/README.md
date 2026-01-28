# 调试脚本说明

本目录包含用于测试和调试Aether平台各个功能的脚本。

## 脚本列表

### 权限系统相关
- **debug_rebac.sh** - 基础ReBAC权限系统测试脚本
- **debug_rebac_full.sh** - 完整的ReBAC权限系统测试脚本，包含递归图遍历测试

### 内容管理相关
- **debug_publish.sh** - 测试文章发布功能
- **debug_publish_rules.sh** - 测试发布规则和流程
- **debug_edit_published.sh** - 测试已发布文章的编辑功能
- **debug_duplicate_title_flow.sh** - 测试重复标题检测流程
- **debug_validation.sh** - 测试内容验证功能

### 知识库相关
- **debug_kb_create.sh** - 测试知识库创建功能
- **debug_kb_list.sh** - 测试知识库列表查询
- **debug_kb_article.sh** - 测试知识库文章操作
- **debug_kb_folder.sh** - 测试知识库文件夹操作
- **debug_kb_move.sh** - 测试知识库内容移动功能

## 使用方法

1. 确保后端服务正在运行（默认端口3000）
2. 根据需要设置环境变量（如API_URL）
3. 运行对应的调试脚本：

```bash
cd Aether/backend/scripts/debug
chmod +x debug_*.sh
./debug_rebac.sh
```

## 注意事项

- 这些脚本主要用于开发和测试环境
- 某些脚本可能需要有效的认证token
- 建议在测试数据库上运行，避免影响生产数据

