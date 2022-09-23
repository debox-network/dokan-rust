use std::{mem, pin::Pin, ptr};

use dokan::{map_win32_error_to_ntstatus, OperationResult};
use winapi::{
	shared::{minwindef, ntdef, ntstatus::*, winerror},
	um::{errhandlingapi::GetLastError, heapapi, securitybaseapi, winnt},
};

fn get_win32_last_error_as_operation_result<T>() -> OperationResult<T> {
	unsafe { Err(map_win32_error_to_ntstatus(GetLastError())) }
}

#[derive(Debug)]
struct PrivateObjectSecurity {
	value: winnt::PSECURITY_DESCRIPTOR,
}

impl PrivateObjectSecurity {
	unsafe fn from_raw(ptr: winnt::PSECURITY_DESCRIPTOR) -> Self {
		Self { value: ptr }
	}
}

impl Drop for PrivateObjectSecurity {
	fn drop(&mut self) {
		unsafe {
			securitybaseapi::DestroyPrivateObjectSecurity(&mut self.value);
		}
	}
}

#[derive(Debug)]
pub struct SecurityDescriptor {
	desc_ptr: winnt::PSECURITY_DESCRIPTOR,
}

unsafe impl Sync for SecurityDescriptor {}

unsafe impl Send for SecurityDescriptor {}

fn get_well_known_sid(sid_type: winnt::WELL_KNOWN_SID_TYPE) -> OperationResult<Box<[u8]>> {
	unsafe {
		let mut sid =
			vec![0u8; mem::size_of::<winnt::SID>() + mem::size_of::<u32>() * 7].into_boxed_slice();
		let mut len = sid.len() as u32;
		let ret = securitybaseapi::CreateWellKnownSid(
			sid_type,
			ptr::null_mut(),
			sid.as_mut_ptr() as winnt::PSID,
			&mut len,
		);
		if ret == minwindef::TRUE {
			Ok(sid)
		} else {
			get_win32_last_error_as_operation_result()
		}
	}
}

fn create_default_dacl() -> OperationResult<Box<[u8]>> {
	unsafe {
		let admins_sid = get_well_known_sid(winnt::WinBuiltinAdministratorsSid)?;
		let system_sid = get_well_known_sid(winnt::WinLocalSystemSid)?;
		let auth_sid = get_well_known_sid(winnt::WinAuthenticatedUserSid)?;
		let users_sid = get_well_known_sid(winnt::WinBuiltinUsersSid)?;

		let acl_len = mem::size_of::<winnt::ACL>()
			+ (mem::size_of::<winnt::ACCESS_ALLOWED_ACE>() - mem::size_of::<u32>()) * 4
			+ admins_sid.len()
			+ system_sid.len()
			+ auth_sid.len()
			+ users_sid.len();
		let mut acl = vec![0u8; acl_len].into_boxed_slice();
		let ret = securitybaseapi::InitializeAcl(
			acl.as_mut_ptr() as winnt::PACL,
			acl_len as u32,
			winnt::ACL_REVISION as u32,
		);
		if ret == minwindef::FALSE {
			return get_win32_last_error_as_operation_result();
		}

		let flags = (winnt::CONTAINER_INHERIT_ACE | winnt::OBJECT_INHERIT_ACE) as u32;
		let ret = securitybaseapi::AddAccessAllowedAceEx(
			acl.as_mut_ptr() as winnt::PACL,
			winnt::ACL_REVISION as u32,
			flags,
			winnt::FILE_ALL_ACCESS,
			admins_sid.as_ptr() as winnt::PSID,
		);
		if ret == minwindef::FALSE {
			return get_win32_last_error_as_operation_result();
		}
		let ret = securitybaseapi::AddAccessAllowedAceEx(
			acl.as_mut_ptr() as winnt::PACL,
			winnt::ACL_REVISION as u32,
			flags,
			winnt::FILE_ALL_ACCESS,
			system_sid.as_ptr() as winnt::PSID,
		);
		if ret == minwindef::FALSE {
			return get_win32_last_error_as_operation_result();
		}
		let ret = securitybaseapi::AddAccessAllowedAceEx(
			acl.as_mut_ptr() as winnt::PACL,
			winnt::ACL_REVISION as u32,
			flags,
			winnt::FILE_GENERIC_READ
				| winnt::FILE_GENERIC_WRITE
				| winnt::FILE_GENERIC_EXECUTE
				| winnt::DELETE,
			auth_sid.as_ptr() as winnt::PSID,
		);
		if ret == minwindef::FALSE {
			return get_win32_last_error_as_operation_result();
		}
		let ret = securitybaseapi::AddAccessAllowedAceEx(
			acl.as_mut_ptr() as winnt::PACL,
			winnt::ACL_REVISION as u32,
			flags,
			winnt::FILE_GENERIC_READ | winnt::FILE_GENERIC_EXECUTE,
			users_sid.as_ptr() as winnt::PSID,
		);
		if ret == minwindef::FALSE {
			return get_win32_last_error_as_operation_result();
		}

		Ok(acl)
	}
}

const FILE_GENERIC_MAPPING: winnt::GENERIC_MAPPING = winnt::GENERIC_MAPPING {
	GenericRead: winnt::FILE_GENERIC_READ,
	GenericWrite: winnt::FILE_GENERIC_WRITE,
	GenericExecute: winnt::FILE_GENERIC_EXECUTE,
	GenericAll: winnt::FILE_ALL_ACCESS,
};

impl SecurityDescriptor {
	pub fn new_inherited(
		parent_desc: &SecurityDescriptor,
		creator_desc: winnt::PSECURITY_DESCRIPTOR,
		token: ntdef::HANDLE,
		is_dir: bool,
	) -> OperationResult<Self> {
		unsafe {
			if !creator_desc.is_null()
				&& securitybaseapi::IsValidSecurityDescriptor(creator_desc) == minwindef::FALSE
			{
				return Err(STATUS_INVALID_PARAMETER);
			}

			let mut priv_desc = ptr::null_mut();
			let ret = securitybaseapi::CreatePrivateObjectSecurity(
				parent_desc.desc_ptr,
				creator_desc,
				&mut priv_desc,
				is_dir as minwindef::BOOL,
				token,
				&FILE_GENERIC_MAPPING as *const _ as *mut _,
			);
			if ret == minwindef::FALSE {
				return get_win32_last_error_as_operation_result();
			}
			let priv_desc = PrivateObjectSecurity::from_raw(priv_desc);

			let heap = heapapi::GetProcessHeap();
			if heap.is_null() {
				return get_win32_last_error_as_operation_result();
			}
			let len = securitybaseapi::GetSecurityDescriptorLength(priv_desc.value) as usize;
			let buf = heapapi::HeapAlloc(heap, 0, len);
			if buf.is_null() {
				return get_win32_last_error_as_operation_result();
			}
			ptr::copy_nonoverlapping(priv_desc.value as *const u8, buf as *mut _, len);
			Ok(Self { desc_ptr: buf })
		}
	}

	pub fn new_default() -> OperationResult<Self> {
		let owner_sid = Pin::new(get_well_known_sid(winnt::WinLocalSystemSid)?);
		let group_sid = Pin::new(get_well_known_sid(winnt::WinLocalSystemSid)?);
		let dacl = Pin::new(create_default_dacl()?);

		unsafe {
			let mut abs_desc = mem::zeroed::<winnt::SECURITY_DESCRIPTOR>();
			let abs_desc_ptr = &mut abs_desc as *mut _ as winnt::PSECURITY_DESCRIPTOR;
			let ret = securitybaseapi::InitializeSecurityDescriptor(
				abs_desc_ptr,
				winnt::SECURITY_DESCRIPTOR_REVISION,
			);
			if ret == minwindef::FALSE {
				return get_win32_last_error_as_operation_result();
			}

			let ret = securitybaseapi::SetSecurityDescriptorOwner(
				abs_desc_ptr,
				owner_sid.as_ptr() as winnt::PSID,
				minwindef::FALSE,
			);
			if ret == minwindef::FALSE {
				return get_win32_last_error_as_operation_result();
			}
			let ret = securitybaseapi::SetSecurityDescriptorGroup(
				abs_desc_ptr,
				group_sid.as_ptr() as winnt::PSID,
				minwindef::FALSE,
			);
			if ret == minwindef::FALSE {
				return get_win32_last_error_as_operation_result();
			}
			let ret = securitybaseapi::SetSecurityDescriptorDacl(
				abs_desc_ptr,
				minwindef::TRUE,
				dacl.as_ptr() as winnt::PACL,
				minwindef::FALSE,
			);
			if ret == minwindef::FALSE {
				return get_win32_last_error_as_operation_result();
			}

			let mut len = 0;
			let ret = securitybaseapi::MakeSelfRelativeSD(abs_desc_ptr, ptr::null_mut(), &mut len);
			let err = GetLastError();
			if ret != minwindef::FALSE || err != winerror::ERROR_INSUFFICIENT_BUFFER {
				return Err(map_win32_error_to_ntstatus(err));
			}

			let heap = heapapi::GetProcessHeap();
			if heap.is_null() {
				return get_win32_last_error_as_operation_result();
			}
			let buf = heapapi::HeapAlloc(heap, 0, len as usize);
			if buf.is_null() {
				return get_win32_last_error_as_operation_result();
			}
			let ret = securitybaseapi::MakeSelfRelativeSD(abs_desc_ptr, buf, &mut len);
			if ret == minwindef::FALSE {
				return get_win32_last_error_as_operation_result();
			}
			Ok(Self { desc_ptr: buf })
		}
	}

	pub fn get_security_info(
		&self,
		sec_info: winnt::SECURITY_INFORMATION,
		sec_desc: winnt::PSECURITY_DESCRIPTOR,
		sec_desc_len: u32,
	) -> OperationResult<u32> {
		unsafe {
			let len = securitybaseapi::GetSecurityDescriptorLength(self.desc_ptr);
			if len > sec_desc_len {
				return Ok(len);
			}

			let mut ret_len = 0;
			let ret = securitybaseapi::GetPrivateObjectSecurity(
				self.desc_ptr,
				sec_info,
				sec_desc,
				sec_desc_len,
				&mut ret_len,
			);
			if ret == minwindef::TRUE {
				Ok(len)
			} else {
				get_win32_last_error_as_operation_result()
			}
		}
	}

	pub fn set_security_info(
		&mut self,
		sec_info: winnt::SECURITY_INFORMATION,
		sec_desc: winnt::PSECURITY_DESCRIPTOR,
	) -> OperationResult<()> {
		unsafe {
			if securitybaseapi::IsValidSecurityDescriptor(sec_desc) == minwindef::FALSE {
				return Err(STATUS_INVALID_PARAMETER);
			}

			let ret = securitybaseapi::SetPrivateObjectSecurityEx(
				sec_info,
				sec_desc,
				&mut self.desc_ptr,
				winnt::SEF_AVOID_PRIVILEGE_CHECK | winnt::SEF_AVOID_OWNER_CHECK,
				&FILE_GENERIC_MAPPING as *const _ as *mut _,
				ptr::null_mut(),
			);
			if ret == minwindef::TRUE {
				Ok(())
			} else {
				get_win32_last_error_as_operation_result()
			}
		}
	}
}

impl Drop for SecurityDescriptor {
	fn drop(&mut self) {
		unsafe {
			heapapi::HeapFree(heapapi::GetProcessHeap(), 0, self.desc_ptr);
		}
	}
}
