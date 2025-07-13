#include "pch.h"

// ��ϵ����ؿ�ƫ��
#define SqlHandleMicroMsgOffset 0x2366934
// ���ں���ؿ�ƫ��
#define SqlHandlePublicMsgOffset 0x239E3C8
// �����¼��ؿ�ƫ��
#define SqlHandleMSGOffset 0x239FF68
// ��ҵ΢����ؿ�ƫ��
#define SqlHandleOpenIMOffset1 0x239E6E0
#define SqlHandleOpenIMOffset2 0x239FF6C
// ����Ȧ���ݿ�ƫ��
#define SqlHandleSnsOffset 0x23A1744
// �ղ����ݿ�ƫ��
#define SqlHandleFavoriteOffset 0x23A03B0

// �������ݿ���Ϣ������
vector<DbInfoStruct> dbs;
map<wstring, DbInfoStruct> dbmap;

/*
 * �������ݿ�����`dbmap`�м������ݿ���
 * dbname�����ݿ���
 * return��DWORD����������ɹ����������ݿ��������򷵻�`0`
 */
DWORD GetDbHandleByDbName(wchar_t *dbname)
{
    if (dbmap.size() == 0)
        GetDbHandles();
    if (dbmap.find(dbname) != dbmap.end())
        return dbmap[dbname].handle;
    return 0;
}

unsigned int GetLocalIdByMsgId(ULONG64 msgid, int &dbIndex)
{
    char sql[260] = {0};
    sprintf_s(sql, "select localId from MSG where MsgSvrID=%llu;", msgid);
    wchar_t dbname[20] = {0};
    for (int i = 0;; i++)
    {
        swprintf_s(dbname, L"MSG%d.db", i);
        DWORD handle = GetDbHandleByDbName(dbname);
        if (handle == 0)
            return 0;
        vector<vector<string>> result = SelectData(handle, (const char *)sql);
        if (result.size() == 0)
            continue;
        dbIndex = dbmap[dbname].extrainfo;
        return stoi(result[1][0]);
    }
    return 0;
}

/*
 * ���ⲿ���õĻ�ȡ���ݿ���Ϣ�ӿ�
 * return��DWORD��`dbs`�׸���Ա��ַ
 */
#ifndef USE_SOCKET
DWORD GetDbHandlesRemote()
{
    if (dbs.size() == 0)
        GetDbHandles();
    return (DWORD)dbs.data();
}
#endif

/*
 * ��ȡ���ݿ���Ϣ�ľ���ʵ��
 * return��void
 */
vector<void *> GetDbHandles()
{
    dbs.clear();
    dbmap.clear();
    DWORD WeChatWinBase = GetWeChatWinBase();
    DWORD SqlHandleBaseAddr = *(DWORD *)(WeChatWinBase + SqlHandleMicroMsgOffset) + 0x1428;
    DWORD SqlHandleBeginAddr = *(DWORD *)SqlHandleBaseAddr;
    DWORD SqlHandleEndAddr = *(DWORD *)(SqlHandleBaseAddr + 0x4);
    DWORD SqlHandlePublicMsgAddr = *(DWORD *)(WeChatWinBase + SqlHandlePublicMsgOffset);
    DWORD SqlHandleMSGAddr = *(DWORD *)(WeChatWinBase + SqlHandleMSGOffset);
    DWORD SqlHandleOpenIMAddr1 = *(DWORD *)(WeChatWinBase + SqlHandleOpenIMOffset1);
    DWORD SqlHandleOpenIMAddr2 = *(DWORD *)(WeChatWinBase + SqlHandleOpenIMOffset2);
    DWORD SqlHandleSnsAddr = *(DWORD *)(WeChatWinBase + SqlHandleSnsOffset);
    DWORD SqlHandleFavoriteAddr = *(DWORD *)(WeChatWinBase + SqlHandleFavoriteOffset);
    vector<DWORD> dbaddrs;
    DWORD dwHandle = 0x0;
    // ��ȡ��ϵ�����ݿ���
    while (SqlHandleBeginAddr < SqlHandleEndAddr)
    {
        dwHandle = *(DWORD *)SqlHandleBeginAddr;
        dbaddrs.push_back(dwHandle);
        SqlHandleBeginAddr += 0x4;
        if (SqlHandleBeginAddr == SqlHandleEndAddr)
            break;
    }
    // ��ȡ���ں����ݿ���
    for (int i = 1; i < 4; i++)
    {
        dwHandle = *((DWORD *)(SqlHandlePublicMsgAddr + i * 0x4));
        dbaddrs.push_back(dwHandle);
    }
    // ��ȡ�����¼���ݿ���
    int msgdb_count = *(int *)(SqlHandleMSGAddr + 0x4);
    DWORD MsgdwHandle = *(DWORD *)(SqlHandleMSGAddr + 0x1C);
    for (int i = 0; i < msgdb_count; i++)
    {
        for (int j = 0; j < 4; j++)
        {
            dwHandle = *(DWORD *)(MsgdwHandle + 0x14 + j * 4);
            dbaddrs.push_back(dwHandle);
        }
        MsgdwHandle += 0x68;
    }
    // ��ȡ��ҵ΢�����ݿ���
    dbaddrs.push_back(*(DWORD *)(SqlHandleOpenIMAddr1 + 0x8));
    for (int i = 0; i < 3; i++)
    {
        dwHandle = *(DWORD *)(SqlHandleOpenIMAddr2 + 0xC + i * 4);
        dbaddrs.push_back(dwHandle);
    }
    // ��ȡ����Ȧ���ݿ���
    dbaddrs.push_back(*(DWORD *)(SqlHandleSnsAddr + 0x64));
    // ��ȡ�ղ����ݿ���
    dbaddrs.push_back(*(DWORD *)(SqlHandleFavoriteAddr + 0x8));

    // ��ȡ���ݿ���Ϣ
    for (auto dbaddr : dbaddrs)
    {
        wstring dbname = wstring((wchar_t *)(*(DWORD *)(dbaddr + 0x50)));
        if (dbmap.find(dbname) != dbmap.end())
            continue;
        DbInfoStruct db = {0};
        wstring tablename((wchar_t *)(*(DWORD *)(dbaddr + 0x64)));
        if (tablename == L"MSG")
        {
            db.extrainfo = *(DWORD *)(dbaddr + 0x17C);
        }
        db.dbname = (wchar_t *)(*(DWORD *)(dbaddr + 0x50));
        db.l_dbname = wcslen(db.dbname);
        db.handle = *(DWORD *)(dbaddr + 0x3C);
        ExecuteSQL(*(DWORD *)(dbaddr + 0x3C), "select * from sqlite_master where type=\"table\";", (DWORD)GetDbInfo, &db);
        dbs.push_back(db);
        dbmap[dbname] = db;
    }

    // ���һ���սṹ�壬��Ϊ��ȡ������־
    DbInfoStruct db_end = {0};
    dbs.push_back(db_end);
#ifdef _DEBUG
    for (unsigned int i = 0; i < dbs.size() - 1; i++)
    {
        printf("dbname = %ws,handle = 0x%08X,table_count:%d\n", dbs[i].dbname, dbs[i].handle, dbs[i].tables.size());
        for (unsigned int j = 0; j < dbs[i].tables.size(); j++)
        {
            cout << "name     = " << dbs[i].tables[j].name << endl;
            cout << "tbl_name = " << dbs[i].tables[j].tbl_name << endl;
            cout << "rootpage = " << dbs[i].tables[j].rootpage << endl;
            cout << "sql      = " << dbs[i].tables[j].sql << endl;
            cout << endl;
        }
        cout << endl;
    }
#endif
    vector<void *> ret_array;
    for (unsigned int i = 0; i < dbs.size() - 1; i++)
        ret_array.push_back(&dbs[i]);
    return ret_array;
}
