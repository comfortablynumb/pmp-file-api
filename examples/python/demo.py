#!/usr/bin/env python3
"""
PMP File API - Interactive Demo

This script demonstrates all major features of the PMP File API.
Run this after starting the API server.

Usage:
    python demo.py

Requirements:
    pip install requests
"""

import sys
import tempfile
from pathlib import Path
from datetime import datetime
from file_api_client import FileAPIClient, FileAPIError


def print_section(title: str):
    """Print formatted section header"""
    print(f"\n{'='*60}")
    print(f" {title}")
    print('='*60)


def demo_basic_operations(api: FileAPIClient, storage: str):
    """Demonstrate basic file operations"""
    print_section("1. Basic File Operations")

    # Create temp file
    with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
        f.write(f"Demo file created at {datetime.now()}\n")
        f.write("This demonstrates basic file operations.\n")
        temp_file = f.name

    try:
        # Upload
        print("\n[Upload] Uploading file...")
        result = api.upload(storage, temp_file,
                          metadata={'demo': True, 'feature': 'basic_ops'})
        print(f"✓ Uploaded: {result['file_name']}")
        print(f"  Size: {result['size']} bytes")
        print(f"  Version: {result['version']}")

        file_name = result['file_name']

        # Get metadata
        print("\n[Metadata] Getting file metadata...")
        metadata = api.get_metadata(storage, file_name)
        print(f"✓ File: {metadata['file_name']}")
        print(f"  Created: {metadata['created_at']}")
        print(f"  Custom metadata: {metadata.get('custom', {})}")

        # Update tags
        print("\n[Tags] Adding tags...")
        api.update_tags(storage, file_name, ['demo', 'test', 'example'])
        print("✓ Tags added: demo, test, example")

        # List files
        print("\n[List] Listing files...")
        files = api.list_files(storage)
        print(f"✓ Found {len(files)} files")
        for f in files[:5]:  # Show first 5
            print(f"  - {f['file_name']} ({f['size']} bytes)")

        # Download
        print("\n[Download] Downloading file...")
        download_path = tempfile.mktemp(suffix='.txt')
        api.download(storage, file_name, download_path)
        print(f"✓ Downloaded to: {download_path}")

        with open(download_path, 'r') as f:
            print(f"  Content: {f.read()[:50]}...")

        # Delete
        print("\n[Delete] Soft deleting file...")
        api.delete(storage, file_name)
        print("✓ File moved to trash")

        # Check trash
        print("\n[Trash] Checking trash...")
        trash = api.list_trash(storage)
        print(f"✓ Files in trash: {len(trash)}")

        # Restore
        print("\n[Restore] Restoring from trash...")
        api.restore_from_trash(storage, file_name)
        print("✓ File restored")

        # Cleanup
        api.delete(storage, file_name)
        api.empty_trash(storage)

        Path(temp_file).unlink()
        if Path(download_path).exists():
            Path(download_path).unlink()

    except FileAPIError as e:
        print(f"✗ Error: {e.message}")


def demo_versioning(api: FileAPIClient, storage: str):
    """Demonstrate file versioning"""
    print_section("2. File Versioning")

    # Create versions
    file_name = 'versioned-document.txt'
    versions = []

    try:
        # Version 1
        print("\n[V1] Creating version 1...")
        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt',
                                        delete=False) as f:
            f.write("Version 1: Initial draft\n")
            f.write(f"Created: {datetime.now()}\n")
            v1_file = f.name

        result = api.upload(storage, v1_file,
                          metadata={'version_label': '1.0', 'status': 'draft'})
        versions.append(result['version_id'])
        print(f"✓ Version 1 created - ID: {result['version_id']}")

        # Version 2
        print("\n[V2] Creating version 2...")
        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt',
                                        delete=False) as f:
            f.write("Version 2: Revised draft\n")
            f.write(f"Updated: {datetime.now()}\n")
            f.write("Added: New content section\n")
            v2_file = f.name

        result = api.create_version(storage, file_name, v2_file)
        versions.append(result['version_id'])
        print(f"✓ Version 2 created - ID: {result['version_id']}")
        print(f"  Parent version: {result['parent_version_id']}")

        # Version 3
        print("\n[V3] Creating version 3 (final)...")
        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt',
                                        delete=False) as f:
            f.write("Version 3: FINAL\n")
            f.write(f"Finalized: {datetime.now()}\n")
            f.write("Status: Approved\n")
            v3_file = f.name

        result = api.create_version(storage, file_name, v3_file)
        versions.append(result['version_id'])
        print(f"✓ Version 3 created - ID: {result['version_id']}")

        # List versions
        print("\n[List] Listing all versions...")
        all_versions = api.list_versions(storage, file_name)
        print(f"✓ Total versions: {len(all_versions)}")
        for v in all_versions:
            print(f"  Version {v['version']}: {v['created_at'][:19]}")

        # Download specific version
        print(f"\n[Download] Downloading version 2...")
        v2_content = api.get_version(storage, file_name, versions[1])
        print("✓ Downloaded version 2:")
        print(f"  {v2_content.decode()[:50]}...")

        # Restore old version
        print(f"\n[Restore] Restoring version 1...")
        result = api.restore_version(storage, file_name, versions[0])
        print(f"✓ Restored - New version {result['version']} created from v1")

        # Cleanup
        api.delete(storage, file_name)
        api.empty_trash(storage)
        for f in [v1_file, v2_file, v3_file]:
            if Path(f).exists():
                Path(f).unlink()

    except FileAPIError as e:
        print(f"✗ Error: {e.message}")


def demo_sharing(api: FileAPIClient, storage: str):
    """Demonstrate file sharing"""
    print_section("3. File Sharing")

    try:
        # Upload file
        print("\n[Upload] Creating file to share...")
        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt',
                                        delete=False) as f:
            f.write("This is a shared file\n")
            f.write("Created for sharing demo\n")
            temp_file = f.name

        result = api.upload(storage, temp_file,
                          metadata={'shareable': True})
        file_name = result['file_name']
        print(f"✓ File uploaded: {file_name}")

        # Create share link
        print("\n[Create Share] Creating password-protected share link...")
        share = api.create_share_link(
            storage, file_name,
            expires_in_seconds=3600,
            max_downloads=3,
            password='demo123'
        )
        print(f"✓ Share link created!")
        print(f"  Link ID: {share['link_id']}")
        print(f"  Expires: {share['expires_at'][:19]}")
        print(f"  Max downloads: {share['max_downloads']}")
        print(f"  Password protected: {share['password_protected']}")

        link_id = share['link_id']

        # Get share info
        print("\n[Info] Getting share link info...")
        info = api.get_share_link(link_id)
        print(f"✓ Downloads: {info['download_count']}/{info['max_downloads']}")

        # Download via share link
        print("\n[Download] Downloading via share link...")
        download_path = tempfile.mktemp(suffix='.txt')
        api.download_shared_file(link_id, password='demo123',
                                output_path=download_path)
        print(f"✓ Downloaded successfully")

        with open(download_path, 'r') as f:
            print(f"  Content: {f.read()}")

        # Revoke share
        print("\n[Revoke] Revoking share link...")
        api.revoke_share_link(link_id)
        print("✓ Share link revoked")

        # Cleanup
        api.delete(storage, file_name)
        api.empty_trash(storage)
        Path(temp_file).unlink()
        if Path(download_path).exists():
            Path(download_path).unlink()

    except FileAPIError as e:
        print(f"✗ Error: {e.message}")


def demo_search(api: FileAPIClient, storage: str):
    """Demonstrate search functionality"""
    print_section("4. Search & Tags")

    try:
        # Upload multiple files with tags
        print("\n[Setup] Uploading files with tags...")
        files_created = []

        file_data = [
            ('financial-report-q4.txt', ['finance', 'q4', '2024', 'report'],
             'Q4 Financial Report\nRevenue and expenses'),
            ('meeting-notes.txt', ['meetings', 'notes', '2024'],
             'Meeting notes from team sync'),
            ('project-plan.txt', ['project', 'planning', 'roadmap'],
             'Project roadmap and milestones'),
        ]

        for fname, tags, content in file_data:
            with tempfile.NamedTemporaryFile(mode='w', suffix='.txt',
                                            delete=False) as f:
                f.write(content)
                temp_file = f.name

            result = api.upload(storage, temp_file, metadata={'type': 'document'})
            api.update_tags(storage, result['file_name'], tags)
            files_created.append(result['file_name'])
            print(f"✓ Uploaded: {result['file_name']} with tags: {tags}")
            Path(temp_file).unlink()

        # Search by query
        print("\n[Search] Searching for 'report'...")
        results = api.search(storage, query='report')
        print(f"✓ Found {results['total']} results")
        for r in results['results']:
            print(f"  - {r['file_name']} (tags: {r['tags']})")

        # Search by tags
        print("\n[Search] Searching by tags ['finance', 'q4']...")
        results = api.search(storage, tags=['finance', 'q4'])
        print(f"✓ Found {results['total']} results")
        for r in results['results']:
            print(f"  - {r['file_name']}")

        # List all tags
        print("\n[Tags] Listing all tags...")
        all_tags = api.list_all_tags(storage)
        print(f"✓ Total unique tags: {len(all_tags)}")
        print(f"  Tags: {', '.join(sorted(all_tags))}")

        # Cleanup
        for fname in files_created:
            api.delete(storage, fname)
        api.empty_trash(storage)

    except FileAPIError as e:
        print(f"✗ Error: {e.message}")


def demo_bulk_operations(api: FileAPIClient, storage: str):
    """Demonstrate bulk operations"""
    print_section("5. Bulk Operations")

    try:
        # Create multiple temp files
        print("\n[Setup] Creating files for bulk upload...")
        temp_files = []
        for i in range(3):
            with tempfile.NamedTemporaryFile(mode='w', suffix=f'-{i}.txt',
                                            delete=False) as f:
                f.write(f"Bulk file {i+1}\n")
                f.write(f"Created for bulk demo\n")
                temp_files.append({
                    'path': f.name,
                    'name': f'bulk-file-{i+1}.txt',
                    'metadata': {'batch': 'demo', 'index': i+1}
                })

        # Bulk upload
        print(f"\n[Bulk Upload] Uploading {len(temp_files)} files...")
        result = api.bulk_upload(storage, temp_files)
        print(f"✓ Bulk upload complete:")
        print(f"  Total: {result['total']}")
        print(f"  Successful: {result['successful']}")
        print(f"  Failed: {result['failed']}")

        file_names = [f['name'] for f in temp_files]

        # Bulk download
        print(f"\n[Bulk Download] Downloading {len(file_names)} files...")
        downloads = api.bulk_download(storage, file_names)
        print(f"✓ Downloaded {len(downloads)} files")
        for dl in downloads:
            print(f"  - {dl['name']} ({len(dl['content'])} chars base64)")

        # Bulk delete
        print(f"\n[Bulk Delete] Deleting {len(file_names)} files...")
        result = api.bulk_delete(storage, file_names)
        print(f"✓ Bulk delete complete:")
        print(f"  Successful: {result['successful']}")

        # Cleanup
        api.empty_trash(storage)
        for f in temp_files:
            if Path(f['path']).exists():
                Path(f['path']).unlink()

    except FileAPIError as e:
        print(f"✗ Error: {e.message}")


def demo_health_monitoring(api: FileAPIClient):
    """Demonstrate health and monitoring"""
    print_section("6. Health & Monitoring")

    try:
        # Basic health
        print("\n[Health] Checking API health...")
        health = api.health()
        print(f"✓ Status: {health['status']}")

        # Comprehensive health
        print("\n[Health All] Checking all storages...")
        health_all = api.health_all()
        print(f"✓ System status: {health_all['status']}")
        print(f"  Uptime: {health_all['uptime_seconds']} seconds")
        print(f"  Storage backends:")
        for name, info in health_all['storages'].items():
            print(f"    - {name}: {info['status']} "
                  f"({info['response_time_ms']}ms)")

        # Cache stats
        print("\n[Cache] Getting cache statistics...")
        stats = api.cache_stats()
        print(f"✓ Cache entries: {stats['entry_count']}")
        print(f"  Size: {stats['weighted_size']} bytes")

    except FileAPIError as e:
        print(f"✗ Error: {e.message}")


def main():
    """Main demo function"""
    print("\n" + "="*60)
    print(" PMP File API - Comprehensive Demo")
    print(" " + "="*58)
    print("\nThis demo will showcase all major features of the API.")
    print("Make sure the API server is running on http://localhost:3000")
    print("="*60)

    # Initialize client
    try:
        api = FileAPIClient('http://localhost:3000')
        storage = 'local-storage'

        # Check if API is available
        try:
            api.health()
        except FileAPIError:
            print("\n✗ Error: Cannot connect to API server")
            print("  Make sure the server is running:")
            print("  $ cargo run --release")
            sys.exit(1)

        # Run demos
        demo_basic_operations(api, storage)
        demo_versioning(api, storage)
        demo_sharing(api, storage)
        demo_search(api, storage)
        demo_bulk_operations(api, storage)
        demo_health_monitoring(api)

        # Summary
        print_section("Demo Complete!")
        print("\n✓ All features demonstrated successfully!")
        print("\nNext steps:")
        print("  1. Check DOCUMENTATION.md for complete API reference")
        print("  2. Explore examples/ directory for more use cases")
        print("  3. Try the file_api_client.py module in your own code")
        print("\nHappy coding!")
        print("="*60 + "\n")

    except Exception as e:
        print(f"\n✗ Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == '__main__':
    main()
