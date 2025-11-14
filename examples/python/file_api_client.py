"""
PMP File API - Python Client Library

A comprehensive Python client for interacting with the PMP File API.
Supports all API features including versioning, sharing, bulk operations, and more.

Usage:
    from file_api_client import FileAPIClient

    api = FileAPIClient('http://localhost:3000')

    # Upload file
    result = api.upload('my-storage', 'document.pdf',
                       metadata={'project': 'alpha'})

    # Search files
    results = api.search('my-storage', query='report',
                        tags=['finance'])
"""

import requests
import base64
import json
from typing import Optional, List, Dict, Any, BinaryIO
from pathlib import Path
import logging

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class FileAPIError(Exception):
    """Custom exception for API errors"""
    def __init__(self, message: str, status_code: Optional[int] = None,
                 response: Optional[Dict] = None):
        self.message = message
        self.status_code = status_code
        self.response = response
        super().__init__(self.message)


class FileAPIClient:
    """
    Python client for PMP File API

    Attributes:
        base_url: Base URL of the API
        session: Requests session for connection pooling
        timeout: Default timeout for requests in seconds
    """

    def __init__(self, base_url: str, api_key: Optional[str] = None,
                 timeout: int = 30):
        """
        Initialize the File API client

        Args:
            base_url: Base URL of the API (e.g., 'http://localhost:3000')
            api_key: Optional API key for authentication
            timeout: Request timeout in seconds (default: 30)
        """
        self.base_url = base_url.rstrip('/')
        self.session = requests.Session()
        self.timeout = timeout

        if api_key:
            self.session.headers.update({'Authorization': f'Bearer {api_key}'})

    def _request(self, method: str, endpoint: str, **kwargs) -> requests.Response:
        """Make HTTP request with error handling"""
        url = f'{self.base_url}{endpoint}'
        kwargs.setdefault('timeout', self.timeout)

        try:
            response = self.session.request(method, url, **kwargs)
            response.raise_for_status()
            return response
        except requests.exceptions.HTTPError as e:
            error_msg = f"HTTP {e.response.status_code}: {e.response.text}"
            raise FileAPIError(error_msg, e.response.status_code,
                             e.response.json() if e.response.content else None)
        except requests.exceptions.RequestException as e:
            raise FileAPIError(f"Request failed: {str(e)}")

    # ========== Basic File Operations ==========

    def upload(self, storage: str, file_path: str,
               metadata: Optional[Dict[str, Any]] = None,
               file_name: Optional[str] = None) -> Dict[str, Any]:
        """
        Upload a file to storage

        Args:
            storage: Storage backend name
            file_path: Path to file to upload
            metadata: Optional custom metadata
            file_name: Optional custom file name (default: use original)

        Returns:
            File metadata dictionary

        Example:
            >>> api.upload('my-storage', 'report.pdf',
            ...           metadata={'project': 'alpha', 'year': 2024})
        """
        path = Path(file_path)
        if not path.exists():
            raise FileAPIError(f"File not found: {file_path}")

        with open(file_path, 'rb') as f:
            files = {'file': (file_name or path.name, f)}
            data = {}
            if metadata:
                data['metadata'] = json.dumps(metadata)

            response = self._request('PUT', f'/api/v1/file/{storage}',
                                   files=files, data=data)
            return response.json()

    def download(self, storage: str, file_name: str,
                output_path: Optional[str] = None) -> bytes:
        """
        Download a file from storage

        Args:
            storage: Storage backend name
            file_name: Name of file to download
            output_path: Optional path to save file (default: return bytes)

        Returns:
            File content as bytes (if output_path not specified)

        Example:
            >>> api.download('my-storage', 'report.pdf', 'local-copy.pdf')
            >>> content = api.download('my-storage', 'data.json')
        """
        response = self._request('GET', f'/api/v1/file/{storage}/{file_name}')

        if output_path:
            with open(output_path, 'wb') as f:
                f.write(response.content)
            logger.info(f"Downloaded to: {output_path}")
            return response.content

        return response.content

    def list_files(self, storage: str, prefix: Optional[str] = None,
                  name_pattern: Optional[str] = None,
                  content_type: Optional[str] = None,
                  tags: Optional[List[str]] = None) -> List[Dict[str, Any]]:
        """
        List files in storage with optional filtering

        Args:
            storage: Storage backend name
            prefix: Filter by path prefix
            name_pattern: Filter by file name pattern
            content_type: Filter by MIME type
            tags: Filter by tags

        Returns:
            List of file metadata dictionaries

        Example:
            >>> files = api.list_files('my-storage',
            ...                       name_pattern='report',
            ...                       tags=['finance'])
        """
        params = {}
        if prefix:
            params['prefix'] = prefix
        if name_pattern:
            params['name_pattern'] = name_pattern
        if content_type:
            params['content_type'] = content_type
        if tags:
            params['tags'] = ','.join(tags)

        response = self._request('GET', f'/api/v1/file/{storage}',
                               params=params)
        return response.json()

    def delete(self, storage: str, file_name: str) -> Dict[str, Any]:
        """
        Soft delete a file (moves to trash)

        Args:
            storage: Storage backend name
            file_name: Name of file to delete

        Returns:
            Success message

        Example:
            >>> api.delete('my-storage', 'old-file.pdf')
        """
        response = self._request('DELETE', f'/api/v1/file/{storage}/{file_name}')
        return response.json()

    def get_metadata(self, storage: str, file_name: str) -> Dict[str, Any]:
        """
        Get file metadata

        Args:
            storage: Storage backend name
            file_name: Name of file

        Returns:
            File metadata dictionary

        Example:
            >>> metadata = api.get_metadata('my-storage', 'report.pdf')
            >>> print(metadata['size'], metadata['created_at'])
        """
        response = self._request('GET',
                               f'/api/v1/file/{storage}/{file_name}/metadata')
        return response.json()

    # ========== Versioning ==========

    def create_version(self, storage: str, file_name: str,
                      file_path: str) -> Dict[str, Any]:
        """
        Create a new version of an existing file

        Args:
            storage: Storage backend name
            file_name: Name of file to version
            file_path: Path to new version content

        Returns:
            New version metadata

        Example:
            >>> api.create_version('docs', 'contract.pdf', 'contract-v2.pdf')
        """
        with open(file_path, 'rb') as f:
            files = {'file': f}
            response = self._request('POST',
                                   f'/api/v1/file/{storage}/{file_name}/versions',
                                   files=files)
            return response.json()

    def list_versions(self, storage: str, file_name: str) -> List[Dict[str, Any]]:
        """
        List all versions of a file

        Args:
            storage: Storage backend name
            file_name: Name of file

        Returns:
            List of version metadata dictionaries

        Example:
            >>> versions = api.list_versions('docs', 'contract.pdf')
            >>> for v in versions:
            ...     print(f"Version {v['version']} - {v['created_at']}")
        """
        response = self._request('GET',
                               f'/api/v1/file/{storage}/{file_name}/versions')
        return response.json()

    def get_version(self, storage: str, file_name: str, version_id: str,
                   output_path: Optional[str] = None) -> bytes:
        """
        Download a specific version of a file

        Args:
            storage: Storage backend name
            file_name: Name of file
            version_id: UUID of version to download
            output_path: Optional path to save file

        Returns:
            Version content as bytes (if output_path not specified)
        """
        response = self._request('GET',
            f'/api/v1/file/{storage}/{file_name}/versions/{version_id}')

        if output_path:
            with open(output_path, 'wb') as f:
                f.write(response.content)
            return response.content

        return response.content

    def restore_version(self, storage: str, file_name: str,
                       version_id: str) -> Dict[str, Any]:
        """
        Restore a previous version (creates new version from old one)

        Args:
            storage: Storage backend name
            file_name: Name of file
            version_id: UUID of version to restore

        Returns:
            New version metadata

        Example:
            >>> api.restore_version('docs', 'contract.pdf', old_version_id)
        """
        response = self._request('POST',
            f'/api/v1/file/{storage}/{file_name}/versions/{version_id}/restore')
        return response.json()

    # ========== File Sharing ==========

    def create_share_link(self, storage: str, file_name: str,
                         expires_in_seconds: int = 3600,
                         max_downloads: Optional[int] = None,
                         password: Optional[str] = None) -> Dict[str, Any]:
        """
        Create a shareable link for a file

        Args:
            storage: Storage backend name
            file_name: Name of file to share
            expires_in_seconds: Link expiration time (default: 1 hour)
            max_downloads: Maximum number of downloads (default: unlimited)
            password: Optional password protection

        Returns:
            Share link information including link_id

        Example:
            >>> share = api.create_share_link('docs', 'report.pdf',
            ...                               expires_in_seconds=86400,
            ...                               password='secret123')
            >>> print(f"Share link ID: {share['link_id']}")
        """
        payload = {
            'file_name': file_name,
            'expires_in_seconds': expires_in_seconds
        }
        if max_downloads is not None:
            payload['max_downloads'] = max_downloads
        if password:
            payload['password'] = password

        response = self._request('POST', f'/api/v1/share/{storage}',
                               json=payload)
        return response.json()

    def get_share_link(self, link_id: str) -> Dict[str, Any]:
        """Get share link information"""
        response = self._request('GET', f'/api/v1/share/{link_id}')
        return response.json()

    def download_shared_file(self, link_id: str, password: Optional[str] = None,
                            output_path: Optional[str] = None) -> bytes:
        """
        Download file using share link

        Args:
            link_id: Share link ID
            password: Password if link is protected
            output_path: Optional path to save file

        Returns:
            File content as bytes (if output_path not specified)
        """
        params = {}
        if password:
            params['password'] = password

        response = self._request('GET', f'/api/v1/share/{link_id}/download',
                               params=params)

        if output_path:
            with open(output_path, 'wb') as f:
                f.write(response.content)
            return response.content

        return response.content

    def revoke_share_link(self, link_id: str) -> Dict[str, Any]:
        """Revoke a share link"""
        response = self._request('DELETE', f'/api/v1/share/{link_id}')
        return response.json()

    # ========== Bulk Operations ==========

    def bulk_upload(self, storage: str,
                   files: List[Dict[str, Any]]) -> Dict[str, Any]:
        """
        Upload multiple files at once

        Args:
            storage: Storage backend name
            files: List of dicts with 'path', 'name', 'metadata'

        Returns:
            Bulk operation result with success/failure counts

        Example:
            >>> result = api.bulk_upload('my-storage', [
            ...     {'path': 'file1.pdf', 'metadata': {'type': 'doc'}},
            ...     {'path': 'file2.jpg', 'metadata': {'type': 'image'}}
            ... ])
            >>> print(f"Uploaded {result['successful']} files")
        """
        encoded_files = []
        for file_info in files:
            file_path = file_info.get('path')
            if not file_path or not Path(file_path).exists():
                logger.warning(f"File not found: {file_path}")
                continue

            with open(file_path, 'rb') as f:
                content = base64.b64encode(f.read()).decode('utf-8')
                encoded_files.append({
                    'name': file_info.get('name', Path(file_path).name),
                    'content': content,
                    'metadata': file_info.get('metadata', {})
                })

        response = self._request('POST', f'/api/v1/bulk/{storage}/upload',
                               json={'files': encoded_files})
        return response.json()

    def bulk_download(self, storage: str,
                     file_names: List[str]) -> List[Dict[str, Any]]:
        """
        Download multiple files at once

        Args:
            storage: Storage backend name
            file_names: List of file names to download

        Returns:
            List of files with base64 encoded content

        Example:
            >>> files = api.bulk_download('my-storage',
            ...                          ['file1.pdf', 'file2.jpg'])
            >>> for file in files:
            ...     content = base64.b64decode(file['content'])
            ...     with open(file['name'], 'wb') as f:
            ...         f.write(content)
        """
        response = self._request('POST', f'/api/v1/bulk/{storage}/download',
                               json={'file_names': file_names})
        return response.json()['files']

    def bulk_delete(self, storage: str,
                   file_names: List[str]) -> Dict[str, Any]:
        """Delete multiple files at once"""
        response = self._request('POST', f'/api/v1/bulk/{storage}/delete',
                               json={'file_names': file_names})
        return response.json()

    # ========== Search & Tags ==========

    def search(self, storage: str, query: Optional[str] = None,
              tags: Optional[List[str]] = None,
              content_type: Optional[str] = None,
              name_pattern: Optional[str] = None,
              include_deleted: bool = False) -> Dict[str, Any]:
        """
        Search files with full-text and filter support

        Args:
            storage: Storage backend name
            query: Search query string
            tags: Filter by tags
            content_type: Filter by MIME type
            name_pattern: Filter by file name pattern
            include_deleted: Include soft-deleted files

        Returns:
            Search results with matching files

        Example:
            >>> results = api.search('docs', query='quarterly report',
            ...                     tags=['finance', 'q4'])
            >>> for file in results['results']:
            ...     print(file['file_name'], file['score'])
        """
        payload = {'include_deleted': include_deleted}
        if query:
            payload['query'] = query
        if tags:
            payload['tags'] = tags
        if content_type:
            payload['content_type'] = content_type
        if name_pattern:
            payload['name_pattern'] = name_pattern

        response = self._request('POST', f'/api/v1/search/{storage}',
                               json=payload)
        return response.json()

    def update_tags(self, storage: str, file_name: str,
                   tags: List[str]) -> Dict[str, Any]:
        """
        Update file tags

        Args:
            storage: Storage backend name
            file_name: Name of file
            tags: New list of tags

        Returns:
            Updated file metadata

        Example:
            >>> api.update_tags('docs', 'report.pdf',
            ...                ['finance', 'q4', 'final'])
        """
        response = self._request('PUT',
                               f'/api/v1/file/{storage}/{file_name}/tags',
                               json={'tags': tags})
        return response.json()

    def list_all_tags(self, storage: str) -> List[str]:
        """List all tags used in storage"""
        response = self._request('GET', f'/api/v1/tags/{storage}')
        return response.json()

    # ========== Trash Management ==========

    def list_trash(self, storage: str) -> List[Dict[str, Any]]:
        """List all soft-deleted files"""
        response = self._request('GET', f'/api/v1/trash/{storage}')
        return response.json()

    def restore_from_trash(self, storage: str, file_name: str) -> Dict[str, Any]:
        """Restore a soft-deleted file"""
        response = self._request('POST',
                               f'/api/v1/trash/{storage}/{file_name}/restore')
        return response.json()

    def empty_trash(self, storage: str) -> Dict[str, Any]:
        """Permanently delete all files in trash"""
        response = self._request('DELETE', f'/api/v1/trash/{storage}')
        return response.json()

    # ========== Health & Monitoring ==========

    def health(self) -> Dict[str, str]:
        """Basic health check"""
        response = self._request('GET', '/health')
        return response.json()

    def health_all(self) -> Dict[str, Any]:
        """Comprehensive health check for all storages"""
        response = self._request('GET', '/health/all')
        return response.json()

    def health_storage(self, storage: str) -> Dict[str, Any]:
        """Health check for specific storage"""
        response = self._request('GET', f'/health/{storage}')
        return response.json()

    def cache_stats(self) -> Dict[str, Any]:
        """Get cache statistics"""
        response = self._request('GET', '/api/v1/cache/stats')
        return response.json()

    def invalidate_cache(self, storage: str, file_name: str) -> Dict[str, Any]:
        """Invalidate cache entry for specific file"""
        response = self._request('DELETE',
                               f'/api/v1/cache/{storage}/{file_name}')
        return response.json()

    def clear_cache(self) -> Dict[str, Any]:
        """Clear all cache entries"""
        response = self._request('DELETE', '/api/v1/cache')
        return response.json()


# Example usage
if __name__ == '__main__':
    # Initialize client
    api = FileAPIClient('http://localhost:3000')

    # Check health
    print("API Health:", api.health())

    # Example operations
    print("\nExample API operations available:")
    print("- api.upload(storage, file_path, metadata)")
    print("- api.download(storage, file_name, output_path)")
    print("- api.search(storage, query, tags)")
    print("- api.create_version(storage, file_name, new_file_path)")
    print("- api.create_share_link(storage, file_name, expires_in_seconds)")
    print("\nSee examples/ directory for complete use cases!")
