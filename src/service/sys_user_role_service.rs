use crate::dao::RB;
use crate::domain::domain::{SysRes, SysUserRole};
use crate::domain::dto::{UserRoleAddDTO, UserRoleEditDTO, UserRolePageDTO};
use crate::domain::vo::{SysResVO, SysRoleVO};
use crate::service::Context;
use chrono::NaiveDateTime;
use rbatis::core::value::DateTimeNow;
use rbatis::core::Result;
use rbatis::crud::CRUD;
use rbatis::plugin::page::{Page, PageRequest};

///用户角色服务
pub struct SysUserRoleService {}

impl SysUserRoleService {
    ///角色分页
    pub async fn page(&self, arg: &UserRolePageDTO) -> Result<Page<SysUserRole>> {
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
    pub async fn add(&self, arg: &UserRoleAddDTO) -> Result<u64> {
        let role = SysUserRole {
            id: Some(
                rbatis::plugin::snowflake::async_snowflake_id()
                    .await
                    .to_string(),
            ),
            user_id: arg.user_id.clone(),
            role_id: arg.role_id.clone(),
            create_date: Some(NaiveDateTime::now()),
        };
        Ok(RB.save("", &role).await?.rows_affected)
    }

    ///角色修改
    pub async fn edit(&self, arg: &UserRoleEditDTO) -> Result<u64> {
        let mut role = SysUserRole {
            id: arg.id.clone(),
            user_id: arg.user_id.clone(),
            role_id: arg.role_id.clone(),
            create_date: None,
        };
        RB.update_by_id("", &mut role).await
    }

    ///角色删除
    pub async fn remove(&self, id: &str) -> Result<u64> {
        RB.remove_by_id::<SysUserRole>("", &id.to_string()).await
    }

    ///角色删除
    pub async fn remove_by_role_id(&self, role_id: &str) -> Result<u64> {
        RB.remove_by_wrapper::<SysUserRole>("", &RB.new_wrapper().eq("role_id", role_id))
            .await
    }

    ///找出角色
    pub async fn find_user_roles(
        &self,
        user_id: &str,
        all_res: &Vec<SysRes>,
    ) -> Result<Vec<SysRoleVO>> {
        if user_id.is_empty() {
            return Ok(vec![]);
        }
        let user_roles = RB
            .fetch_list_by_wrapper::<SysUserRole>("", &RB.new_wrapper().eq("user_id", user_id))
            .await?;
        let role_ids = &fields!(&user_roles, role_id);
        let roles = Context.sys_role_service.finds(role_ids).await?;
        let res_map = Context.sys_res_service.to_hash_map(all_res)?;
        let role_res_vec = Context
            .sys_role_service
            .find_role_res(&fields!(&user_roles, role_id))
            .await?;

        let mut role_vos = vec![];
        for role in roles {
            //load res
            let mut resources = vec![];
            for role_res in &role_res_vec {
                if role.id.is_some() && role.id.eq(&role_res.role_id) {
                    let res = res_map.get(role_res.res_id.as_ref().unwrap_or(&String::new()));
                    match res {
                        Some(res) => {
                            resources.push(SysResVO::from(*res));
                        }
                        _ => {}
                    }
                }
            }
            role_vos.push(SysRoleVO {
                id: role.id,
                name: role.name,
                parent_id: role.parent_id,
                del: role.del,
                create_date: role.create_date,
                resources: resources,
            });
        }
        return Ok(role_vos);
    }
}
