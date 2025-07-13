#pragma once
#include <windows.h>
using namespace std;

// ����Hook�����ݺ󣬸����̷߳����ź�
#define WM_WAIT_HOOK_DATA WM_USER + 0x1

/*
 * ΢���еĻ������ݽṹ
 * buffer��UNICODE�ַ���
 * length��`buffer`�ַ���
 * maxLength��`buffer`����ַ���
 * fill1��ռλ��Ա1��Ĭ��Ϊ0
 * fill2��ռλ��Ա2��Ĭ��Ϊ0
 * WxString��Ĭ�Ϲ��캯��
 */
typedef struct WxStringW
{
    wchar_t *buffer = NULL;
    DWORD length = 0;
    DWORD maxLength = 0;
    DWORD fill1 = 0;
    DWORD fill2 = 0;
    WxStringW() {}
    WxStringW(wstring &str)
    {
        buffer = (wchar_t *)str.c_str();
        length = str.length();
        maxLength = str.length() * 2;
    }
    WxStringW(const wchar_t *pStr)
    {
        buffer = (wchar_t *)pStr;
        length = wcslen(pStr);
        maxLength = wcslen(pStr) * 2;
    }
    WxStringW(int tmp)
    {
        buffer = NULL;
        length = 0x0;
        maxLength = 0x0;
    }
    WxStringW(wchar_t *pStr)
    {
        buffer = pStr;
        length = wcslen(pStr);
        maxLength = wcslen(pStr) * 2;
    }
    void set_value(const wchar_t *pStr)
    {
        buffer = (wchar_t *)pStr;
        length = wcslen(pStr);
        maxLength = wcslen(pStr) * 2;
    }
} WxString;

/*
 * ���浥����Ϣ�Ľṹ
 * messagetype����Ϣ����
 * sender��������wxid
 * wxid�����sender��Ⱥ��id����˳�Ա������巢����wxid��������`sender`һ��
 * message����Ϣ���ݣ����ı���Ϣ��xml��ʽ
 * filepath��ͼƬ���ļ���������Դ�ı���·��
 */
struct ReceiveMsgStruct
{
    DWORD pid = 0;
    DWORD messagetype = 0;
    BOOL isSendMessage = 0;
    unsigned long long msgid = 0;
    wstring sender;
    wstring wxid;
    wstring message;
    wstring filepath;
    wstring time;
    wstring extrainfo;
    ~ReceiveMsgStruct()
    {
    }
};

// vector���ڴ��еı�����ʽ
struct VectorStruct
{
#ifdef _DEBUG
    DWORD v_head;
#endif
    DWORD v_data;
    DWORD v_end1;
    DWORD v_end2;
};

struct UserInfo
{
    int errcode;
    wchar_t *keyword;
    int l_keyword;
    wchar_t *v3;
    int l_v3;
    wchar_t *NickName;
    int l_NickName;
    wchar_t *Signature;
    int l_Signature;
    wchar_t *v2;
    int l_v2;
    wchar_t *Nation;
    int l_Nation;
    wchar_t *Province;
    int l_Province;
    wchar_t *City;
    int l_City;
    wchar_t *BigAvatar;
    int l_BigAvatar;
    wchar_t *SmallAvatar;
    int l_SmallAvatar;
    DWORD sex;
    BOOL over;
};

/*
 * �������ݿⵥ������Ϣ�Ľṹ��
 * name��������l_name��`name`�ַ���
 * tbl_name��������l_tbl_name��`tbl_name`�ַ���
 * sql��������䣻l_sql��`sql`�ַ���
 * rootpage�����ţ�l_rootpage��`rootpage`�ַ���
 */
struct TableInfoStruct
{
    char *name;
    DWORD l_name;
    char *tbl_name;
    DWORD l_tbl_name;
    char *sql;
    DWORD l_sql;
    char *rootpage;
    DWORD l_rootpage;
};

/*
 * �������ݿ���Ϣ�Ľṹ��
 * handle�����ݿ���
 * dbname�����ݿ���
 * l_dbname��`dbname`�ַ���
 * tables������������б���Ϣ������
 * count�����б������
 */
struct DbInfoStruct
{
    DWORD handle = 0;
    wchar_t *dbname = NULL;
    DWORD l_dbname = 0;
    vector<TableInfoStruct> tables;
    DWORD count = 0;
    DWORD extrainfo = 0;
};

/*
 * ���浥��������Ϣ�Ľṹ��
 * wxIdAddr��wxid�����ַ
 * wxNumberAddr��΢�źű����ַ
 * wxNickNameAddr���ǳƱ����ַ
 * wxRemarkAddr����ע�����ַ
 * WxFriendStructW��Ĭ�Ϲ��캯��
 */
struct WxFriendStruct
{
    DWORD wxIdAddr;
    DWORD wxNumberAddr;
    DWORD wxNickNameAddr;
    DWORD wxRemarkAddr;
    DWORD wxTypeAddr;
    DWORD wxVerifyFlagAddr;
    WxFriendStruct(DWORD wxIdAddr, DWORD wxNumberAddr,
                   DWORD wxNickNameAddr, DWORD wxRemarkAddr,
                   DWORD wxTypeAddr, DWORD wxVerfifyFlagAddr)
    {
        this->wxIdAddr = wxIdAddr;
        this->wxNumberAddr = wxNumberAddr;
        this->wxNickNameAddr = wxNickNameAddr;
        this->wxRemarkAddr = wxRemarkAddr;
        this->wxTypeAddr = wxTypeAddr;
        this->wxVerifyFlagAddr = wxVerfifyFlagAddr;
    }
};

typedef struct CHAT_MSGTag
{
    DWORD handle1 = 0;
    DWORD null_value1 = 0;
    ULONG64 sequence = 0x0;
    DWORD null_value2[2] = {0};
    ULONG64 msgsequence = 0x0;
    DWORD localId = 0;
    DWORD null_value3[3] = {0};
    ULONG64 msgid = 0x0;
    DWORD type = 0x1;
    DWORD isSendMsg = 0x1;
    DWORD unknown_value1 = 0x2;
    DWORD create_time = 0x0;
    WxString takler = {0};
    WxString null_string1 = {0};
    WxString content = {0};
    DWORD null_value4[2] = {0};
    DWORD extrabuf = 0;
    DWORD extrabuf_len = 0;
    DWORD null_value5[17] = {0};
    BOOL isSyncMsg = 0x1;
    DWORD null_value6[21] = {0};
    DWORD handle2 = 0;
    DWORD handle3 = 0;
    void *unknown_ptr1 = NULL;
    DWORD null_value7[13] = {0};
    WxString chatroom_member = {0};
    WxString md5 = {0};
    WxString thumbnail = {0};
    WxString file_save_path = {0};
    DWORD null_value8[11] = {0};
    WxString extra_info = {0};
    DWORD null_value9[16] = {0};
    DWORD unknown_value2 = 0x1;
    DWORD unknown_value3 = 0x1;
    DWORD null_value10[7] = {0};
    DWORD unknown_value4 = 0x1;
    DWORD null_value11 = 0;
    DWORD unknown_value5 = 0xFF;
    DWORD unknown_value6 = 0x1;
    DWORD null_value12[6] = {0};
    void *unknown_ptr2 = NULL;
    DWORD null_value13[2] = {0};
} CHAT_MSG, *PCHAT_MSG;

struct WxStringA
{
    char buffer[0x10] = {0};
    int length;
    int maxLength;
    WxStringA(string &str)
    {
        this->length = str.length();
        this->maxLength = this->length - (this->length % 0x10) + 0xF;
        if (this->length == 0)
        {
            *(DWORD *)this->buffer = 0;
        }
        else if (this->length < 0x10)
        {
            memcpy(this->buffer, str.c_str(), this->length + 1);
        }
        else
        {
            *(DWORD *)this->buffer = (DWORD)str.c_str();
        }
    }
    WxStringA(const char *buf)
    {
        this->length = strlen(buf);
        this->maxLength = this->length - (this->length % 0x10) + 0xF;
        if (this->length == 0)
        {
            *(DWORD *)this->buffer = 0;
        }
        else if (this->length < 0x10)
        {
            memcpy(this->buffer, buf, this->length + 1);
        }
        else
        {
            *(DWORD *)this->buffer = (DWORD)buf;
        }
    }
    char *get()
    {
        if (this->length < 0x10)
        {
            return this->buffer;
        }
        return (char *)(*(DWORD *)this->buffer);
    }
};
