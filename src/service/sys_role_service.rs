use rbatis::core::Result;
use rbatis::crud::CRUD;
use rbatis::plugin::page::{Page, PageRequest};

use crate::dao::RB;
use crate::domain::domain::{SysRes, SysRole, SysRoleRes, SysUserRole};
use crate::domain::dto::{RoleAddDTO, RoleEditDTO, RolePageDTO};
use crate::service::Context;
use chrono::NaiveDateTime;
use rbatis::core::value::DateTimeNow;

///角色服务
pub struct SysRoleService {}

impl SysRoleService {
    ///角色分页
    pub async fn page(&self, arg: &RolePageDTO) -> Result<Page<SysRole>> {
        let wrapper = RB.new_wrapper();
        let data = RB
            .fetch_page_by_wrapper(
                "",
                &wrapper,
                &PageRequest::new(arg.page.unwrap_or(0), arg.size.unwrap_or(0)),
            )
            .await?;
        return Ok(data);
    }

    ///角色添加
    pub async fn add(&self, arg: &RoleAddDTO) -> Result<u64> {
        let role = SysRole {
            id: Some(
                rbatis::plugin::snowflake::async_snowflake_id()
                    .await
                    .to_string(),
            ),
            name: arg.name.clone(),
            parent_id: arg.parent_id.clone(),
            del: Some(0),
            create_date: Some(NaiveDateTime::now()),
        };
        Ok(RB.save("", &role).await?.rows_affected)
    }

    ///角色修改
    pub async fn edit(&self, arg: &RoleEditDTO) -> Result<u64> {
        let mut role = SysRole {
            id: arg.id.clone(),
            name: arg.name.clone(),
            parent_id: arg.parent_id.clone(),
            del: None,
            create_date: None,
        };
        RB.update_by_id("", &mut role).await
    }

    ///角色删除
    pub async fn remove(&self, arg: &str) -> Result<u64> {
        RB.remove_by_id::<SysRole>("", &arg.to_string()).await
    }

    ///角色删除,同时删除用户关系，权限关系
    pub async fn remove_role_relation(&self, id: &str) -> Result<u64> {
        let remove_roles = self.remove(id).await?;
        //删除关联关系
        let remove_user_roles = Context.sys_user_role_service.remove_by_role_id(id).await?;
        let remove_role_res = Context.sys_role_res_service.remove_by_role_id(id).await?;
        return Ok(remove_roles + remove_user_roles + remove_role_res);
    }

    pub async fn finds(&self, ids: &Vec<String>) -> Result<Vec<SysRole>> {
        RB.fetch_list_by_wrapper("", &RB.new_wrapper().r#in("id", ids))
            .await
    }

    pub async fn find_role_res(&self, ids: &Vec<String>) -> Result<Vec<SysRoleRes>> {
        RB.fetch_list_by_wrapper("", &RB.new_wrapper().r#in("role_id", ids))
            .await
    }

    pub async fn find_user_permission(
        &self,
        user_id: &str,
        all_res: &Vec<SysRes>,
    ) -> Result<Vec<String>> {
        let user_roles: Vec<SysUserRole> = RB
            .fetch_list_by_wrapper("", &RB.new_wrapper().eq("user_id", user_id))
            .await?;
        let role_res = self.find_role_res(&fields!(&user_roles, role_id)).await?;
        let res = Context
            .sys_res_service
            .finds_layer(&fields!(&role_res, res_id), &all_res)
            .await?;
        let mut permissons = vec![];
        for item in res {
            permissons.push(item.permission.clone().unwrap_or_default());
        }
        return Ok(permissons);
    }
}
