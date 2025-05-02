use dokan_sys::DOKAN_IO_SECURITY_CONTEXT;
use widestring::U16CStr;
use winapi::{
	shared::{ntdef::NTSTATUS, ntstatus::STATUS_NOT_IMPLEMENTED},
	um::winnt::{ACCESS_MASK, PSECURITY_DESCRIPTOR},
};

use crate::data::{
	CreateFileInfo, DiskSpaceInfo, FileInfo, FileTimeOperation, FillDataResult, FindData,
	FindStreamData, OperationInfo, VolumeInfo,
};

/// Returned by [`FileSystemHandler`]'s methods.
pub type OperationResult<T> = Result<T, NTSTATUS>;

/// Handles operations for a mounted file system.
///
/// Dokan invokes the callback functions in this trait to handle file system operations. These
/// functions have similar semantics to that of corresponding Windows API functions.
///
/// Implementation of most callback functions can be omitted by returning `Err(`[`STATUS_NOT_IMPLEMENTED`]`)`
/// if the corresponding feature is not supported. To make things flexible, all of the functions are
/// provided with a default implementation which is a no-op and returns `Err(`[`STATUS_NOT_IMPLEMENTED`]`)`
/// (except [`cleanup`] and [`close_file`] which don't have return values). However, omitting the
/// implementation of some important callbacks such as [`create_file`] will make the file system
/// unusable.
///
/// `Err` type is [`NTSTATUS`]. Use [`map_win32_error_to_ntstatus`] to convert from Win32 errors
/// (e.g. returned by [`GetLastError`]).
///
/// [`cleanup`]: Self::cleanup
/// [`close_file`]: Self::close_file
/// [`create_file`]: Self::create_file
/// [`map_win32_error_to_ntstatus`]: crate::map_win32_error_to_ntstatus
/// [`GetLastError`]: winapi::um::errhandlingapi::GetLastError
#[allow(unused_variables)]
pub trait FileSystemHandler<'c, 'h: 'c>: Sync + Sized + 'h {
	/// Type of the context associated with an open file object.
	type Context: Sync + 'c;

	/// Called when a file object is created.
	///
	/// The flags p-them to flags accepted by [`CreateFile`] using the
	/// [`map_kernel_to_user_create_file_flags`] helper function.
	///
	/// [`ZwCreateFile`]: https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-zwcreatefile
	/// [`CreateFile`]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew
	/// [`map_kernel_to_user_create_file_flags`]: crate::map_kernel_to_user_create_file_flags
	#[allow(clippy::too_many_arguments)]
	fn create_file(
		&'h self,
		file_name: &U16CStr,
		security_context: &DOKAN_IO_SECURITY_CONTEXT,
		desired_access: ACCESS_MASK,
		file_attributes: u32,
		share_access: u32,
		create_disposition: u32,
		create_options: u32,
		info: &mut OperationInfo<'c, 'h, Self>,
	) -> OperationResult<CreateFileInfo<Self::Context>> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Called when the last handle for the file object has been closed.
	///
	/// If [`info.delete_pending`] returns `true`, the file should be deleted in this function. As the function doesn't
	/// have a return value, you should make sure the file is deletable in [`delete_file`] or [`delete_directory`].
	///
	/// Note that the file object hasn't been released and there might be more I/O operations before
	/// [`close_file`] gets called. (This typically happens when the file is memory-mapped.)
	///
	/// Normally [`close_file`] will be called shortly after this function. However, the file object
	/// may also be reused, and in that case [`create_file`] will be called instead.
	///
	/// [`info.delete_pending`]: OperationInfo::delete_pending
	/// [`delete_file`]: Self::delete_file
	/// [`delete_directory`]: Self::delete_directory
	/// [`close_file`]: Self::close_file
	/// [`create_file`]: Self::create_file
	fn cleanup(
		&'h self,
		file_name: &U16CStr,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) {
	}

	/// Called when the last handle for the handle object has been closed and released.
	///
	/// This is the last function called during the lifetime of the file object. You can safely
	/// release any resources allocated for it (such as file handles, buffers, etc.). The associated
	/// [`context`] object will also be dropped once this function returns. In case the file object is
	/// reused and thus this function isn't called, the [`context`] will be dropped before
	/// [`create_file`] gets called.
	///
	/// [`context`]: Self::Context
	/// [`create_file`]: Self::create_file
	fn close_file(
		&'h self,
		file_name: &U16CStr,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) {
	}

	/// Reads data from the file.
	///
	/// The number of bytes that actually gets read should be returned.
	///
	/// See [`ReadFile`] for more information.
	///
	/// [`ReadFile`]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-readfile
	fn read_file(
		&'h self,
		file_name: &U16CStr,
		offset: i64,
		buffer: &mut [u8],
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<u32> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Writes data to the file.
	///
	/// The number of bytes that actually gets written should be returned.
	///
	/// If [`info.write_to_eof`] returns `true`, data should be written to the end of file and the
	/// `offset` parameter should be ignored.
	///
	/// See [`WriteFile`] for more information.
	///
	/// [`info.write_to_eof`]: OperationInfo::write_to_eof
	/// [`WriteFile`]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-writefile
	fn write_file(
		&'h self,
		file_name: &U16CStr,
		offset: i64,
		buffer: &[u8],
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<u32> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Flushes the buffer of the file and causes all buffered data to be written to the file.
	///
	/// See [`FlushFileBuffers`] for more information.
	///
	/// [`FlushFileBuffers`]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-flushfilebuffers
	fn flush_file_buffers(
		&'h self,
		file_name: &U16CStr,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<()> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Gets information about the file.
	///
	/// See [`GetFileInformationByHandle`] for more information.
	///
	/// [`GetFileInformationByHandle`]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfileinformationbyhandle
	fn get_file_information(
		&'h self,
		file_name: &U16CStr,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<FileInfo> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Lists all child items in the directory.
	///
	/// `fill_find_data` should be called for every child item in the directory.
	///
	/// It will only be called if [`find_files_with_pattern`] returns [`STATUS_NOT_IMPLEMENTED`].
	///
	/// See [`FindFirstFile`] for more information.
	///
	/// [`find_files_with_pattern`]: Self::find_files_with_pattern
	/// [`FindFirstFile`]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findfirstfilew
	fn find_files(
		&'h self,
		file_name: &U16CStr,
		fill_find_data: impl FnMut(&FindData) -> FillDataResult,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<()> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Lists all child items that matches the specified `pattern` in the directory.
	///
	/// `fill_find_data` should be called for every matching child item in the directory.
	///
	/// [`is_name_in_expression`] can be used to determine if a file name matches the pattern.
	///
	/// If this function returns [`STATUS_NOT_IMPLEMENTED`], [`find_files`] will be called instead and
	/// pattern matching will be handled directly by Dokan.
	///
	/// See [`FindFirstFile`] for more information.
	///
	/// [`is_name_in_expression`]: crate::is_name_in_expression
	/// [`find_files`]: Self::find_files
	/// [`FindFirstFile`]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findfirstfilew
	fn find_files_with_pattern(
		&'h self,
		file_name: &U16CStr,
		pattern: &U16CStr,
		fill_find_data: impl FnMut(&FindData) -> FillDataResult,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<()> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Sets attributes of the file.
	///
	/// `file_attributes` can be combination of one or more [file attribute constants] defined by
	/// Windows.
	///
	/// See [`SetFileAttributes`] for more information.
	///
	/// [file attribute constants]: https://docs.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants
	/// [`SetFileAttributes`]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-setfileattributesw
	fn set_file_attributes(
		&'h self,
		file_name: &U16CStr,
		file_attributes: u32,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<()> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Sets the time when the file was created, last accessed and last written.
	///
	/// See [`SetFileTime`] for more information.
	///
	/// [`SetFileTime`]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-setfiletime
	fn set_file_time(
		&'h self,
		file_name: &U16CStr,
		creation_time: FileTimeOperation,
		last_access_time: FileTimeOperation,
		last_write_time: FileTimeOperation,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<()> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Checks if the file can be deleted.
	///
	/// The file should not be deleted in this function. Instead, it should only check if the file
	/// can be deleted and return `Ok` if that is possible.
	///
	/// It will also be called with [`info.delete_pending`] returning `false` to notify that the
	/// file is no longer requested to be deleted.
	///
	/// [`info.delete_pending`]: OperationInfo::delete_pending
	fn delete_file(
		&'h self,
		file_name: &U16CStr,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<()> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Checks if the directory can be deleted.
	///
	/// Similar to [`delete_file`], it should only check if the directory can be deleted and delay
	/// the actual deletion to the [`cleanup`] function.
	///
	/// It will also be called with [`info.delete_pending`] returning `false` to notify that the
	/// directory is no longer requested to be deleted.
	///
	/// [`delete_file`]: Self::delete_file
	/// [`cleanup`]: Self::cleanup
	/// [`info.delete_pending`]: OperationInfo::delete_pending
	fn delete_directory(
		&'h self,
		file_name: &U16CStr,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<()> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Moves the file.
	///
	/// If the `new_file_name` already exists, the function should only replace the existing file
	/// when `replace_if_existing` is `true`, otherwise it should return appropriate error.
	///
	/// Note that renaming is a special kind of moving and is also handled by this function.
	///
	/// See [`MoveFileEx`] for more information.
	///
	/// [`MoveFileEx`]: https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw
	fn move_file(
		&'h self,
		file_name: &U16CStr,
		new_file_name: &U16CStr,
		replace_if_existing: bool,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<()> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Sets end-of-file position of the file.
	///
	/// The `offset` value is zero-based, so it actually refers to the offset to the byte
	/// immediately following the last valid byte in the file.
	///
	/// See [`FILE_END_OF_FILE_INFORMATION`] for more information.
	///
	/// [`FILE_END_OF_FILE_INFORMATION`]: https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddk/ns-ntddk-_file_end_of_file_information
	fn set_end_of_file(
		&'h self,
		file_name: &U16CStr,
		offset: i64,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<()> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Sets allocation size of the file.
	///
	/// The allocation size is the number of bytes allocated in the underlying physical device for
	/// the file.
	///
	/// See [`FILE_ALLOCATION_INFORMATION`] for more information.
	///
	/// [`FILE_ALLOCATION_INFORMATION`]: https://docs.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_allocation_information
	fn set_allocation_size(
		&'h self,
		file_name: &U16CStr,
		alloc_size: i64,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<()> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Locks the file for exclusive access.
	///
	/// It will only be called if [`MountFlags::FILELOCK_USER_MODE`] was specified when mounting the
	/// volume, otherwise Dokan will take care of file locking.
	///
	/// See [`LockFile`] for more information.
	///
	/// [`MountFlags::FILELOCK_USER_MODE`]: crate::MountFlags::FILELOCK_USER_MODE
	/// [`LockFile`]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-lockfile
	fn lock_file(
		&'h self,
		file_name: &U16CStr,
		offset: i64,
		length: i64,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<()> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Unlocks the previously locked file.
	///
	/// It will only be called if [`MountFlags::FILELOCK_USER_MODE`] was specified when mounting the
	/// volume, otherwise Dokan will take care of file locking.
	///
	/// See [`UnlockFile`] for more information.
	///
	/// [`MountFlags::FILELOCK_USER_MODE`]: crate::MountFlags::FILELOCK_USER_MODE
	/// [`UnlockFile`]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-unlockfile
	fn unlock_file(
		&'h self,
		file_name: &U16CStr,
		offset: i64,
		length: i64,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<()> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Gets free space information about the disk.
	///
	/// See [`GetDiskFreeSpaceEx`] for more information.
	///
	/// [`GetDiskFreeSpaceEx`]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getdiskfreespaceexw
	fn get_disk_free_space(
		&'h self,
		info: &OperationInfo<'c, 'h, Self>,
	) -> OperationResult<DiskSpaceInfo> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Gets information about the volume and file system.
	///
	/// See [`GetVolumeInformation`] for more information.
	///
	/// [`GetVolumeInformation`]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getvolumeinformationbyhandlew
	fn get_volume_information(
		&'h self,
		info: &OperationInfo<'c, 'h, Self>,
	) -> OperationResult<VolumeInfo> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Called when Dokan has successfully mounted the volume.
	fn mounted(
		&'h self,
		mount_point: &U16CStr,
		info: &OperationInfo<'c, 'h, Self>,
	) -> OperationResult<()> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Called when Dokan is unmounting the volume.
	fn unmounted(&'h self, info: &OperationInfo<'c, 'h, Self>) -> OperationResult<()> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Gets security information of a file.
	///
	/// Size of the security descriptor in bytes should be returned on success. If the buffer is not
	/// large enough, the number should still be returned, and [`STATUS_BUFFER_OVERFLOW`] will be
	/// automatically passed to Dokan if it is larger than `buffer_length`.
	///
	/// See [`GetFileSecurity`] for more information.
	///
	/// [`STATUS_BUFFER_OVERFLOW`]: winapi::shared::ntstatus::STATUS_BUFFER_OVERFLOW
	/// [`GetFileSecurity`]: https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getfilesecuritya
	fn get_file_security(
		&'h self,
		file_name: &U16CStr,
		security_information: u32,
		security_descriptor: PSECURITY_DESCRIPTOR,
		buffer_length: u32,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<u32> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Sets security information of a file.
	///
	/// See [`SetFileSecurity`] for more information.
	///
	/// [`SetFileSecurity`]: https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-setfilesecuritya
	fn set_file_security(
		&'h self,
		file_name: &U16CStr,
		security_information: u32,
		security_descriptor: PSECURITY_DESCRIPTOR,
		buffer_length: u32,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<()> {
		Err(STATUS_NOT_IMPLEMENTED)
	}

	/// Lists all alternative streams of the file.
	///
	/// `fill_find_stream_data` should be called for every stream of the file, including the default
	/// data stream `::$DATA`.
	///
	/// See [`FindFirstStream`] for more information.
	///
	/// [`FindFirstStream`]: https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findfirststreamw
	fn find_streams(
		&'h self,
		file_name: &U16CStr,
		fill_find_stream_data: impl FnMut(&FindStreamData) -> FillDataResult,
		info: &OperationInfo<'c, 'h, Self>,
		context: &'c Self::Context,
	) -> OperationResult<()> {
		Err(STATUS_NOT_IMPLEMENTED)
	}
}
